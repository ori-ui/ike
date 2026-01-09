use std::{ffi, ptr};

use ike_core::{Padding, Size};
use jni::{JNIEnv, objects::JObject};
use raw_window_handle::{AndroidNdkWindowHandle, DisplayHandle, RawWindowHandle, WindowHandle};

use crate::{Event, EventLoop, Window, WindowState, context::Proxy, send_event};

#[derive(Debug)]
pub(super) enum WindowEvent {
    Created(*mut ndk_sys::ANativeWindow),
    Destroyed,
    Redraw,
    Resized,
    FocusChanged(bool),
    InsetsChanged {
        system_bars: Padding,
        ime:         Padding,
        cutout:      Padding,
    },
    RenderFinished,
}

unsafe impl Send for WindowEvent {}

impl<'a, T> EventLoop<'a, T> {
    pub fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Created(android) => {
                let WindowState::Pending {
                    id,
                    ref mut updates,
                } = self.window
                else {
                    tracing::error!("android only supports one window");
                    return;
                };

                let window_handle = unsafe {
                    let Some(window) = ptr::NonNull::new(android) else {
                        tracing::error!("android window was null");
                        return;
                    };

                    WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(
                        AndroidNdkWindowHandle::new(window.cast()),
                    ))
                };

                let width = unsafe { ndk_sys::ANativeWindow_getWidth(android) };
                let height = unsafe { ndk_sys::ANativeWindow_getHeight(android) };

                tracing::debug!(width, height, "window created");

                let surface = unsafe {
                    ike_skia::vulkan::Surface::new(
                        &mut self.vulkan,
                        DisplayHandle::android(),
                        window_handle,
                        width as u32,
                        height as u32,
                    )
                };

                let surface = match surface {
                    Ok(surface) => surface,
                    Err(err) => {
                        tracing::error!("failed creating vulkan context: {err}");
                        return;
                    }
                };

                let mut window = Window {
                    id,
                    android,
                    surface,
                    focused: false,
                    width: width as u32,
                    height: height as u32,
                    insets: Padding::all(0.0),
                };

                for update in updates.drain(..) {
                    window.handle_update(update);
                }

                self.window = WindowState::Open(window);

                let size = Size::new(
                    width as f32 / self.scale_factor,
                    height as f32 / self.scale_factor,
                );

                if let Some(id) = id {
                    (self.context.world).window_scaled(id, size, self.scale_factor);
                }
            }

            WindowEvent::Destroyed => {
                match self.window {
                    WindowState::Open(ref window) => {
                        self.window = WindowState::Pending {
                            id:      window.id,
                            updates: Vec::new(),
                        };
                    }

                    WindowState::Pending { .. } => {}
                };
            }

            WindowEvent::Redraw => {
                if let WindowState::Open(ref mut window) = self.window {
                    if self.is_rendering {
                        self.wants_render = true;
                        return;
                    }

                    let Some(id) = window.id else {
                        return;
                    };

                    if let Some(animate) = self.animate.take() {
                        let delta_time = animate.elapsed();
                        self.context.world.animate(id, delta_time);
                    }

                    let Some(win) = self.context.world.get_window(id) else {
                        tracing::error!("window not registered with ike");
                        return;
                    };

                    if let Some(choreographer) = self.choreographer {
                        unsafe {
                            ndk_sys::AChoreographer_postFrameCallback(
                                choreographer.as_ptr(),
                                Some(frame_callback),
                                (&mut self.proxy) as *mut _ as *mut _,
                            );
                        };
                    }

                    self.is_rendering = true;

                    let result = window.surface.draw(
                        &mut self.painter,
                        win.color(),
                        self.scale_factor,
                        || {},
                        |canvas| self.context.world.draw(id, canvas),
                    );

                    if let Err(err) = result {
                        tracing::error!("draw failed: {err}");
                    }
                }
            }

            WindowEvent::Resized => {
                tracing::trace!("window resized");

                if let WindowState::Open(ref mut window) = self.window {
                    let width = unsafe { ndk_sys::ANativeWindow_getWidth(window.android) };
                    let height = unsafe { ndk_sys::ANativeWindow_getHeight(window.android) };

                    window.width = width as u32;
                    window.height = height as u32;

                    if let Err(err) = window.surface.resize(width as u32, height as u32) {
                        tracing::error!("surface resize failed: {err}");
                        return;
                    }

                    let Some(id) = window.id else {
                        return;
                    };

                    let size = Size::new(
                        width as f32 / self.scale_factor,
                        height as f32 / self.scale_factor,
                    );

                    self.context.world.window_resized(id, size);
                }
            }

            WindowEvent::InsetsChanged {
                system_bars,
                ime,
                cutout,
            } => {
                tracing::trace!(
                    ?system_bars,
                    ?ime,
                    ?cutout,
                    "insets changed",
                );

                if let WindowState::Open(ref mut window) = self.window {
                    let mut insets = Padding::all(0.0);

                    insets.left += system_bars.left + ime.left + cutout.left;
                    insets.top += system_bars.top + ime.top + cutout.top;
                    insets.right += system_bars.right + ime.right + cutout.right;
                    insets.bottom += system_bars.bottom.max(ime.bottom) + cutout.bottom;

                    insets.left /= self.scale_factor;
                    insets.top /= self.scale_factor;
                    insets.right /= self.scale_factor;
                    insets.bottom /= self.scale_factor;

                    window.insets = insets;

                    if let Some(id) = window.id {
                        (self.context.world).window_inset(id, insets);
                    }
                }
            }

            WindowEvent::FocusChanged(focused) => {
                tracing::trace!(focused, "window focus changed");

                if let WindowState::Open(ref mut window) = self.window {
                    window.focused = focused;

                    let Some(id) = window.id else {
                        return;
                    };

                    self.context.world.window_focused(id, focused);
                }
            }

            WindowEvent::RenderFinished => {
                tracing::trace!("render finished");

                self.is_rendering = false;

                if self.wants_render {
                    self.wants_render = false;
                    self.proxy.send(Event::Window(WindowEvent::Redraw));
                }
            }
        }
    }
}

unsafe extern "C" fn frame_callback(_frame_time_nanos: i64, data: *mut ffi::c_void) {
    let proxy = unsafe { &*data.cast::<Proxy>() };
    proxy.send(Event::Window(
        WindowEvent::RenderFinished,
    ));
}

pub unsafe extern "C" fn on_apply_window_insets<'local>(
    _env: JNIEnv<'local>,
    _rust_view: JObject<'local>,
    system_bars_left: i32,
    system_bars_top: i32,
    system_bars_right: i32,
    system_bars_bottom: i32,
    ime_left: i32,
    ime_top: i32,
    ime_right: i32,
    ime_bottom: i32,
    cutout_left: i32,
    cutout_top: i32,
    cutout_right: i32,
    cutout_bottom: i32,
) {
    let system_bars = Padding {
        left:   system_bars_left as f32,
        top:    system_bars_top as f32,
        right:  system_bars_right as f32,
        bottom: system_bars_bottom as f32,
    };

    let ime = Padding {
        left:   ime_left as f32,
        top:    ime_top as f32,
        right:  ime_right as f32,
        bottom: ime_bottom as f32,
    };

    let cutout = Padding {
        left:   cutout_left as f32,
        top:    cutout_top as f32,
        right:  cutout_right as f32,
        bottom: cutout_bottom as f32,
    };

    send_event(Event::Window(
        WindowEvent::InsetsChanged {
            system_bars,
            ime,
            cutout,
        },
    ));
}

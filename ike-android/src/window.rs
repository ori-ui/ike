use std::{ffi, ptr};

use ike_core::Size;
use raw_window_handle::{AndroidNdkWindowHandle, DisplayHandle, RawWindowHandle, WindowHandle};

use crate::{Event, EventLoop, Window, WindowState, context::Proxy};

#[derive(Debug)]
pub(super) enum WindowEvent {
    Created(*mut ndk_sys::ANativeWindow),
    Destroyed,
    Redraw,
    Resized,
    FocusChanged(bool),
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
                    self.context.window_scaled(id, self.scale_factor, size);
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
                        self.context.animate(id, delta_time);
                    }

                    let Some(win) = self.context.get_window(id) else {
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
                        |canvas| self.context.draw(id, canvas),
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

                    self.context.window_resized(id, size);
                }
            }

            WindowEvent::FocusChanged(focused) => {
                tracing::trace!(focused, "window focus changed");

                if let WindowState::Open(ref mut window) = self.window {
                    window.focused = focused;

                    let Some(id) = window.id else {
                        return;
                    };

                    self.context.window_focused(id, focused);
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

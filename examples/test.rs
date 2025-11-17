use std::{ffi, mem, num::NonZeroU32};

use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use ike::{
    Point, PointerMoveEvent, Size, Tree, WidgetId,
    skia::{SkiaCanvas, SkiaFonts},
    widgets::{Aligned, Button, Label},
};
use skia_safe::{Color, gpu::gl::FramebufferInfo};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowId},
};

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut tree = Tree::new();
    let mut cx = tree.build();

    let label = Label::new(&mut cx, "wahoo");
    let button = Button::new(&mut cx, label);
    let aligned = Aligned::new(&mut cx, 0.5, 0.5, button);

    let mut state = State {
        window: None,
        fonts: SkiaFonts::new(),
        tree,
        root: aligned.upcast(),
    };

    event_loop.run_app(&mut state).unwrap();
}

struct State {
    window: Option<GlWindow>,
    fonts: SkiaFonts,
    tree: Tree,
    root: WidgetId,
}

struct GlWindow {
    gl_config: glutin::config::Config,
    fb_info: FramebufferInfo,
    skia_surface: skia_safe::Surface,
    gl_surface: Surface<WindowSurface>,
    skia_context: skia_safe::gpu::DirectContext,
    gl_context: PossiblyCurrentContext,
    window: Window,
}

impl ApplicationHandler for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (_, gl_config) = DisplayBuilder::new()
            .build(event_loop, ConfigTemplateBuilder::new(), |configs| {
                configs
                    .reduce(|acc, cfg| match cfg.num_samples() > acc.num_samples() {
                        true => cfg,
                        false => acc,
                    })
                    .unwrap()
            })
            .unwrap();

        let window =
            glutin_winit::finalize_window(event_loop, Window::default_attributes(), &gl_config)
                .unwrap();

        let handle = window.window_handle().unwrap().as_raw();

        let attrs = ContextAttributesBuilder::new().build(Some(handle));

        let gl_context = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &attrs)
                .unwrap()
        };

        let size = window.inner_size();

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            handle,
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        );

        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let gl_context = gl_context.make_current(&gl_surface).unwrap();

        let interface = skia_safe::gpu::gl::Interface::new_load_with_cstr(|name| {
            if name == c"eglGetCurrentDisplay" {
                return std::ptr::null();
            }

            gl_config.display().get_proc_address(name)
        })
        .unwrap();

        let mut skia_context = skia_safe::gpu::direct_contexts::make_gl(interface, None).unwrap();

        let fb_info = unsafe {
            let mut fboid: ffi::c_int = 0;

            let get_integerv = gl_config.display().get_proc_address(c"glGetIntegerv");
            assert!(!get_integerv.is_null());

            let get_integerv: unsafe extern "C" fn(ffi::c_uint, *mut ffi::c_int) =
                mem::transmute(get_integerv);

            get_integerv(0x8CA6, &mut fboid);

            FramebufferInfo {
                fboid: fboid as u32,
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            }
        };

        let render_target = skia_safe::gpu::backend_render_targets::make_gl(
            (size.width as i32, size.height as i32),
            gl_config.num_samples() as usize,
            gl_config.stencil_size() as usize,
            fb_info,
        );

        let skia_surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
            &mut skia_context,
            &render_target,
            skia_safe::gpu::SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap();

        self.window = Some(GlWindow {
            gl_config,
            fb_info,
            skia_surface,
            gl_surface,
            skia_context,
            gl_context,
            window,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(ref window) = self.window {
                    let logical = position.to_logical(window.window.scale_factor());

                    let event = PointerMoveEvent {
                        position: Point::new(logical.x, logical.y),
                        id: None,
                    };

                    self.tree.pointer_event(
                        &mut self.fonts,
                        &self.root,
                        &ike::PointerEvent::Move(event),
                    );

                    if self.tree.needs_draw(&self.root) {
                        window.window.request_redraw();
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(ref mut window) = self.window {
                    window.gl_context.make_current(&window.gl_surface).unwrap();

                    let size = window
                        .window
                        .inner_size()
                        .to_logical(window.window.scale_factor());

                    self.tree.layout(
                        &mut self.fonts,
                        &self.root,
                        Size::new(size.width, size.height),
                    );

                    let canvas = window.skia_surface.canvas();
                    canvas.clear(Color::WHITE);
                    canvas.save();
                    canvas.scale((
                        window.window.scale_factor() as f32,
                        window.window.scale_factor() as f32,
                    ));

                    {
                        let mut canvas = SkiaCanvas::new(canvas, &mut self.fonts);
                        self.tree.draw(&self.root, &mut canvas);

                        if self.tree.needs_draw(&self.root) {
                            window.window.request_redraw();
                        }
                    }

                    canvas.restore();

                    window.skia_context.flush_and_submit();
                    window.gl_surface.swap_buffers(&window.gl_context).unwrap();
                }
            }

            WindowEvent::Resized(size) => {
                if let Some(ref mut window) = self.window {
                    let render_target = skia_safe::gpu::backend_render_targets::make_gl(
                        (size.width as i32, size.height as i32),
                        window.gl_config.num_samples() as usize,
                        window.gl_config.stencil_size() as usize,
                        window.fb_info,
                    );

                    let skia_surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
                        &mut window.skia_context,
                        &render_target,
                        skia_safe::gpu::SurfaceOrigin::BottomLeft,
                        skia_safe::ColorType::RGBA8888,
                        None,
                        None,
                    )
                    .unwrap();

                    window.skia_surface = skia_surface;
                    window.window.request_redraw();

                    window.gl_surface.resize(
                        &window.gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );
                }
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => {}
        }
    }
}

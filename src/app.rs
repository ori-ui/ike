use tracing_subscriber::{EnvFilter, layer::SubscriberExt};

use crate::{Context, Effect};

pub struct App {}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init_log() {
        let mut filter = EnvFilter::default();

        if cfg!(debug_assertions) {
            filter = filter.add_directive(tracing::Level::DEBUG.into());
        }

        if let Ok(env) = std::env::var("RUST_LOG")
            && let Ok(env) = env.parse()
        {
            filter = filter.add_directive(env);
        }

        let subscriber = tracing_subscriber::registry().with(filter);

        #[cfg(not(target_arch = "wasm32"))]
        let subscriber = subscriber.with(tracing_subscriber::fmt::layer());

        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    pub fn run<T, V>(self, data: &mut T, mut ui: impl FnMut(&mut T) -> V + 'static)
    where
        V: Effect<T> + 'static,
        V::State: 'static,
    {
        Self::init_log();

        let build: UiBuilder<T> = Box::new(move |data| Box::new(ui(data)));
        crate::winit::run(data, build)
    }
}

pub(crate) type AnyEffect<T> = Box<dyn ori::AnyView<Context, T, ori::NoElement>>;
pub(crate) type UiBuilder<T> = Box<dyn FnMut(&mut T) -> AnyEffect<T>>;

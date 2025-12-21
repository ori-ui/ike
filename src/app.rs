use tracing_subscriber::{EnvFilter, layer::SubscriberExt};

use crate::Effect;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[cfg(backend = "winit")]
    #[error(transparent)]
    Winit(ike_winit::Error),

    #[cfg(backend = "android")]
    #[error(transparent)]
    Android(ike_android::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

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

    pub fn install_log() {
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

        let subscriber = {
            #[cfg(not(target_os = "android"))]
            let fmt_layer = tracing_subscriber::fmt::layer();

            #[cfg(target_os = "android")]
            let fmt_layer = tracing_subscriber::fmt::layer() // android uses it's own logging
                .with_writer(ike_android::MakeAndroidWriter);

            subscriber.with(fmt_layer)
        };

        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    pub fn run<T, V>(self, data: &mut T, mut ui: impl FnMut(&mut T) -> V + 'static) -> Result<()>
    where
        V: Effect<T> + 'static,
        V::State: 'static,
    {
        Self::install_log();

        let build: ike_ori::UiBuilder<T> = Box::new(move |data| Box::new(ui(data)));

        #[cfg(backend = "winit")]
        ike_winit::run(data, build).map_err(Error::Winit)?;

        #[cfg(backend = "android")]
        ike_android::run(data, build).map_err(Error::Android)?;

        Ok(())
    }
}

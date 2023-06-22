use {
    crate::{metrics::Metrics, stores::example::ExampleStoreArc, Configuration},
    build_info::BuildInfo,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Configuration,
    pub build_info: BuildInfo,
    pub metrics: Option<Metrics>,
    pub example_store: ExampleStoreArc,
}

build_info::build_info!(fn build_info);

impl AppState {
    pub fn new(config: Configuration, example_store: ExampleStoreArc) -> crate::Result<AppState> {
        let build_info: &BuildInfo = build_info();

        Ok(AppState {
            config,
            build_info: build_info.clone(),
            metrics: None,
            example_store,
        })
    }

    pub fn set_metrics(&mut self, metrics: Metrics) {
        self.metrics = Some(metrics);
    }
}

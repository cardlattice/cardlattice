use prometheus::{Encoder, HistogramOpts, HistogramVec, IntCounterVec, Registry, TextEncoder};

#[derive(Clone)]
pub struct Metrics {
    pub requests: IntCounterVec,
    pub latency: HistogramVec,
    pub database_latency: HistogramVec,
    registry: Registry,
}

impl Default for Metrics {
    fn default() -> Self {
        let registry = Registry::new();
        let requests = IntCounterVec::new(
            prometheus::Opts::new(
                "cardlattice_http_requests_total",
                "HTTP requests by route template and status",
            ),
            &["route", "method", "status"],
        )
        .expect("valid metric");
        let latency = HistogramVec::new(
            HistogramOpts::new(
                "cardlattice_http_duration_seconds",
                "Origin response latency",
            )
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
            ]),
            &["route"],
        )
        .expect("valid metric");
        let database_latency = HistogramVec::new(
            HistogramOpts::new(
                "cardlattice_database_duration_seconds",
                "Database operation latency",
            ),
            &["operation"],
        )
        .expect("valid metric");
        registry
            .register(Box::new(requests.clone()))
            .expect("new registry");
        registry
            .register(Box::new(latency.clone()))
            .expect("new registry");
        registry
            .register(Box::new(database_latency.clone()))
            .expect("new registry");
        Self {
            requests,
            latency,
            database_latency,
            registry,
        }
    }
}
impl Metrics {
    pub fn render(&self) -> String {
        let mut buffer = Vec::new();
        TextEncoder::new()
            .encode(&self.registry.gather(), &mut buffer)
            .expect("valid metrics");
        String::from_utf8(buffer).expect("UTF-8 metrics")
    }
}
pub fn init() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
}

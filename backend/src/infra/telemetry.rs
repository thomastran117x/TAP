use tracing_subscriber::EnvFilter;

pub fn init(filter: &str) {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .init();
}

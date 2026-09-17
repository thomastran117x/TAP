use serde::Serialize;

#[derive(Serialize)]
pub(super) struct Health {
    pub status: &'static str,
}

#[derive(Serialize)]
pub(super) struct Readiness {
    pub status: &'static str,
    pub postgres: bool,
    pub redis: bool,
    pub opensearch: bool,
}

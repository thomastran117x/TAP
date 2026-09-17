#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tap_backend::app::run().await
}

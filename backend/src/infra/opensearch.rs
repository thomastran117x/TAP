use anyhow::{Context, Result};
use opensearch::{
    OpenSearch,
    auth::Credentials,
    http::{
        Url,
        transport::{SingleNodeConnectionPool, TransportBuilder},
    },
};

use std::time::Duration;

pub async fn connect(
    url: &Url,
    credentials: Option<&(String, String)>,
    timeout: Duration,
) -> Result<OpenSearch> {
    let pool = SingleNodeConnectionPool::new(url.clone());
    let mut transport = TransportBuilder::new(pool).timeout(timeout);
    if let Some((username, password)) = credentials {
        transport = transport.auth(Credentials::Basic(username.clone(), password.clone()));
    }
    let client = OpenSearch::new(transport.build().context("Invalid OpenSearch transport")?);
    client
        .ping()
        .send()
        .await
        .context("Unable to connect to OpenSearch")?
        .error_for_status_code()
        .context("OpenSearch rejected the connection")?;
    Ok(client)
}

pub async fn ping(client: &OpenSearch) -> Result<()> {
    client.ping().send().await?.error_for_status_code()?;
    Ok(())
}

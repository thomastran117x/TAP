use std::{future::Future, time::Duration};

use super::models::Readiness;
use crate::{app::AppState, infra};

pub(super) async fn readiness(state: &AppState) -> Readiness {
    let (postgres, redis, opensearch) = tokio::join!(
        check(
            "postgres",
            state.dependency_timeout,
            infra::postgres::ping(&state.postgres)
        ),
        check(
            "redis",
            state.dependency_timeout,
            infra::redis::ping(&state.redis)
        ),
        check(
            "opensearch",
            state.dependency_timeout,
            infra::opensearch::ping(&state.opensearch)
        ),
    );
    Readiness {
        status: if postgres && redis && opensearch {
            "ready"
        } else {
            "unavailable"
        },
        postgres,
        redis,
        opensearch,
    }
}

async fn check(
    name: &str,
    timeout: Duration,
    future: impl Future<Output = anyhow::Result<()>>,
) -> bool {
    match tokio::time::timeout(timeout, future).await {
        Ok(Ok(())) => true,
        Ok(Err(error)) => {
            tracing::warn!(service = name, %error, "Readiness check failed");
            false
        }
        Err(_) => {
            tracing::warn!(service = name, "Readiness check timed out");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::check;

    #[tokio::test]
    async fn failed_dependency_is_unavailable() {
        assert!(
            !check("test", std::time::Duration::from_secs(3), async {
                anyhow::bail!("offline")
            })
            .await
        );
    }
}

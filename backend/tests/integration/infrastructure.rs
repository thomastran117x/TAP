use anyhow::{Result, ensure};
use opensearch::{
    IndexParts, SearchParts,
    indices::{IndicesCreateParts, IndicesDeleteParts},
    params::Refresh,
};
use serde_json::{Value, json};
use sqlx::Acquire;

use super::support;

#[tokio::test]
#[ignore = "requires the integration test services"]
async fn postgres_round_trip_uses_bound_values_and_rolls_back() -> Result<()> {
    let (_, state) = support::connect().await?;
    let mut connection = state.postgres.acquire().await?;
    let mut transaction = connection.begin().await?;
    sqlx::query("CREATE TEMP TABLE integration_values (value TEXT NOT NULL) ON COMMIT DROP")
        .execute(&mut *transaction)
        .await?;
    let input = "value'); DROP TABLE integration_values; --";
    sqlx::query("INSERT INTO integration_values (value) VALUES ($1)")
        .bind(input)
        .execute(&mut *transaction)
        .await?;
    let stored: String = sqlx::query_scalar("SELECT value FROM integration_values")
        .fetch_one(&mut *transaction)
        .await?;
    transaction.rollback().await?;
    let table: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('pg_temp.integration_values')::text")
            .fetch_one(&mut *connection)
            .await?;
    ensure!(stored == input, "Postgres did not preserve the bound value");
    ensure!(table.is_none(), "The test table survived rollback");
    drop(connection);
    state.postgres.close().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires the integration test services"]
async fn redis_round_trip_has_expiration_and_cleans_up() -> Result<()> {
    let (_, state) = support::connect().await?;
    let mut connection = state.redis.clone();
    let key = support::unique_name("redis");
    let result = tokio::time::timeout(state.dependency_timeout, async {
        redis::cmd("SET")
            .arg(&key)
            .arg("integration-value")
            .arg("EX")
            .arg(60)
            .query_async::<()>(&mut connection)
            .await?;
        let value: String = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut connection)
            .await?;
        let ttl: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut connection)
            .await?;
        ensure!(
            value == "integration-value",
            "Redis returned an unexpected value"
        );
        ensure!(
            ttl > 0 && ttl <= 60,
            "Redis key must have a bounded expiration"
        );
        Ok::<(), anyhow::Error>(())
    })
    .await
    .unwrap_or_else(|error| Err(error.into()));
    // Cleanup runs even when round-trip assertions fail; expiration is a fallback.
    let cleanup = tokio::time::timeout(
        state.dependency_timeout,
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<u64>(&mut connection),
    )
    .await;
    state.postgres.close().await;
    result?;
    cleanup??;
    Ok(())
}

#[tokio::test]
#[ignore = "requires the integration test services"]
async fn opensearch_indexes_searches_and_deletes_an_isolated_index() -> Result<()> {
    let (_, state) = support::connect().await?;
    let index = support::unique_name("search");
    let result: Result<()> = async {
        state
            .opensearch
            .indices()
            .create(IndicesCreateParts::Index(&index))
            .body(json!({
                "settings": {"number_of_shards": 1, "number_of_replicas": 0},
                "mappings": {"properties": {"marker": {"type": "keyword"}}}
            }))
            .send()
            .await?
            .error_for_status_code()?;
        state
            .opensearch
            .index(IndexParts::IndexId(&index, "1"))
            .refresh(Refresh::WaitFor)
            .body(json!({"marker": &index}))
            .send()
            .await?
            .error_for_status_code()?;
        let response: Value = state
            .opensearch
            .search(SearchParts::Index(&[&index]))
            .body(json!({"query": {"term": {"marker": &index}}}))
            .send()
            .await?
            .error_for_status_code()?
            .json()
            .await?;
        ensure!(
            response["hits"]["total"]["value"] == 1,
            "OpenSearch did not return the indexed document"
        );
        ensure!(
            response["hits"]["hits"][0]["_source"]["marker"] == index,
            "OpenSearch returned unexpected document data"
        );
        Ok(())
    }
    .await;
    // Delete only this test's index, including when indexing/searching fails.
    let cleanup = state
        .opensearch
        .indices()
        .delete(IndicesDeleteParts::Index(&[&index]))
        .send()
        .await;
    state.postgres.close().await;
    result?;
    cleanup?.error_for_status_code()?;
    Ok(())
}

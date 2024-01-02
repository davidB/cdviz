use crate::errors::Result;
use crate::settings::DbSettings;
use crate::Event;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::{info, Instrument};

/// Build database connections pool
///
/// # Errors
///
/// Fail if we cannot connect to the database
pub(crate) async fn build_pool(settings: &DbSettings) -> Result<PgPool> {
    let pool_options = PgPoolOptions::new()
        .min_connections(settings.pool_connections_min)
        .max_connections(settings.pool_connections_max);
    info!(
        max_connections = pool_options.get_max_connections(),
        min_connections = pool_options.get_min_connections(),
        acquire_timeout = ?pool_options.get_acquire_timeout(),
        idle_timeout = ?pool_options.get_idle_timeout(),
        max_lifetime = ?pool_options.get_max_lifetime(),
        test_before_acquire = pool_options.get_test_before_acquire(),
        "Using the database"
    );

    let pool = pool_options.connect(&settings.url).await?;

    Ok(pool)
}

// basic handmade span far to be compliant with
//[opentelemetry-specification/.../database.md](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.22.0/specification/trace/semantic_conventions/database.md)
#[allow(dead_code)]
fn build_otel_span(db_operation: &str) -> tracing::Span {
    tracing::trace_span!(
        target: tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET,
        "DB request",
        db.system = "postgresql",
        // db.statement = stmt,
        db.operation = db_operation,
        otel.name = db_operation, // should be <db.operation> <db.name>.<db.sql.table>,
        otel.kind = "CLIENT",
        otel.status_code = tracing::field::Empty,
    )
}

// store event as json in db (postgresql using sqlx)
pub(crate) async fn store_event(pg_pool: &PgPool, event: Event) -> Result<()> {
    // let query = r#"
    //     INSERT INTO events (at, data)
    //     VALUES ($1, $2)
    // "#;

    // sqlx::query(query)
    //     .bind(event.timestamp)
    //     .bind(event.raw)
    //     .execute(&mut tx)
    //     .await?;

    sqlx::query!(
        r#"
        INSERT INTO events (timestamp, raw)
        VALUES ($1, $2)
        "#,
        event.timestamp,
        event.raw
    )
    .execute(pg_pool)
    .instrument(build_otel_span("INSERT INTO events"))
    .await?;

    Ok(())
}

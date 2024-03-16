use crate::{errors::Result, Message};

use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{info, Instrument};

use super::Sink;

/// The database client config
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    /// The database url (with username, password and the database)
    //        default_value = "postgresql://postgres:passwd@localhost:5432/cdviz"
    url: String,

    /// The minimum number of connections to the database to maintain at all times.
    /// minimum > 0, require to have access to the database at startup time,
    /// consume a little more resource on idle
    /// and could increase performance on low load (keep prepared statement,...)
    // https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.min_connections
    // default_value = "1"
    pool_connections_min: u32,

    /// The maximum number of connections to the database to open / to maintain.
    // https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.max_connections
    //        default_value = "10"
    pool_connections_max: u32,
}

/// Build database connections pool
///
/// # Errors
///
/// Fail if we cannot connect to the database
impl TryFrom<Config> for DbSink {
    type Error = crate::errors::Error;

    fn try_from(config: Config) -> Result<Self> {
        let pool_options = PgPoolOptions::new()
            .min_connections(config.pool_connections_min)
            .max_connections(config.pool_connections_max);
        info!(
            max_connections = pool_options.get_max_connections(),
            min_connections = pool_options.get_min_connections(),
            acquire_timeout = ?pool_options.get_acquire_timeout(),
            idle_timeout = ?pool_options.get_idle_timeout(),
            max_lifetime = ?pool_options.get_max_lifetime(),
            test_before_acquire = pool_options.get_test_before_acquire(),
            "Using the database"
        );

        let pool = pool_options.connect_lazy(&config.url)?;

        Ok(Self { pool })
    }
}

pub(crate) struct DbSink {
    pool: PgPool,
}

impl Sink for DbSink {
    async fn send(&self, message: &Message) -> Result<()> {
        store_event(
            &self.pool,
            // TODO build Event from raw json
            Event {
                timestamp: message.received_at,
                raw: serde_json::to_value(&message.cdevent)?,
            },
        )
        .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct Event {
    timestamp: OffsetDateTime,
    raw: serde_json::Value,
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
async fn store_event(pg_pool: &PgPool, event: Event) -> Result<()> {
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
        INSERT INTO cdevents_lake (timestamp, payload)
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

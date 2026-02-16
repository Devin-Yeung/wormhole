use async_trait::async_trait;
use jiff::Timestamp;
use sqlx::{MySqlPool, Row};
use wormhole_core::error::StorageError;
use wormhole_core::repository::{ReadRepository, Repository, Result, UrlRecord};
use wormhole_core::shortcode::ShortCode;

/// MySQL implementation of the repository contract.
///
/// Soft delete is implemented with `deleted_at`. Reads only return active
/// records (`deleted_at IS NULL` and not expired). Inserts never reuse an
/// existing short code, including soft-deleted rows, to preserve analytics
/// history with a single-row-per-code model.
#[derive(Debug, Clone)]
pub struct MySqlRepository {
    pool: MySqlPool,
}

impl MySqlRepository {
    /// Creates a repository from an existing MySQL connection pool.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Creates a repository by opening a new MySQL connection pool.
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = MySqlPool::connect(database_url)
            .await
            .map_err(map_sqlx_error)?;
        Ok(Self::new(pool))
    }

    /// Returns a reference to the underlying pool.
    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }
}

fn now_unix_seconds() -> i64 {
    Timestamp::now().as_second()
}

fn parse_expire_at(seconds: Option<i64>) -> Result<Option<Timestamp>> {
    seconds
        .map(|value| {
            Timestamp::from_second(value).map_err(|e| {
                StorageError::InvalidData(format!("invalid expire_at timestamp '{}': {e}", value))
            })
        })
        .transpose()
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
}

fn map_sqlx_error(err: sqlx::Error) -> StorageError {
    let message = err.to_string();

    match err {
        sqlx::Error::PoolTimedOut => StorageError::Timeout(message),
        sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed
        | sqlx::Error::Io(_)
        | sqlx::Error::Tls(_) => StorageError::Unavailable(message),
        sqlx::Error::ColumnIndexOutOfBounds { .. }
        | sqlx::Error::ColumnNotFound(_)
        | sqlx::Error::ColumnDecode { .. }
        | sqlx::Error::TypeNotFound { .. }
        | sqlx::Error::Decode(_)
        | sqlx::Error::RowNotFound => StorageError::InvalidData(message),
        _ => StorageError::Query(message),
    }
}

#[async_trait]
impl ReadRepository for MySqlRepository {
    async fn get(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        let now = now_unix_seconds();

        let row = sqlx::query(
            r#"
            SELECT original_url, expire_at
            FROM short_urls
            WHERE short_code = ?
              AND deleted_at IS NULL
              AND (expire_at IS NULL OR expire_at > ?)
            LIMIT 1
            "#,
        )
        .bind(code.as_str())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let original_url: String = row.try_get("original_url").map_err(map_sqlx_error)?;
        let expire_at_raw: Option<i64> = row.try_get("expire_at").map_err(map_sqlx_error)?;
        let expire_at = parse_expire_at(expire_at_raw)?;

        Ok(Some(UrlRecord {
            original_url,
            expire_at,
        }))
    }

    async fn exists(&self, code: &ShortCode) -> Result<bool> {
        let exists = sqlx::query(
            r#"
            SELECT 1
            FROM short_urls
            WHERE short_code = ?
            LIMIT 1
            "#,
        )
        .bind(code.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .is_some();

        Ok(exists)
    }
}

#[async_trait]
impl Repository for MySqlRepository {
    async fn insert(&self, code: &ShortCode, record: UrlRecord) -> Result<()> {
        let expire_at = record.expire_at.map(|ts| ts.as_second());

        let result = sqlx::query(
            r#"
            INSERT INTO short_urls (short_code, original_url, expire_at, deleted_at)
            VALUES (?, ?, ?, NULL)
            "#,
        )
        .bind(code.as_str())
        .bind(record.original_url)
        .bind(expire_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) if is_unique_violation(&err) => Err(StorageError::Conflict(code.to_string())),
            Err(err) => Err(map_sqlx_error(err)),
        }
    }

    async fn delete(&self, code: &ShortCode) -> Result<bool> {
        let now = now_unix_seconds();

        let result = sqlx::query(
            r#"
            UPDATE short_urls
            SET deleted_at = ?
            WHERE short_code = ?
              AND deleted_at IS NULL
            "#,
        )
        .bind(now)
        .bind(code.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected() > 0)
    }
}

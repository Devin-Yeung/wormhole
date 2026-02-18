use std::time::Duration;

use jiff::{SignedDuration, Timestamp};
use sqlx::mysql::MySqlPoolOptions;
use wormhole_core::{ShortCode, UrlRecord};
use wormhole_storage::{MySqlRepository, ReadRepository, Repository, StorageError};
use wormhole_test_infra::mysql::{MySqlServer, MysqlConfig};

struct Fixture {
    _mysql: MySqlServer,
    repo: MySqlRepository,
}

impl Fixture {
    async fn start() -> Self {
        let mysql = MySqlServer::new(MysqlConfig::builder().build())
            .await
            .expect("start mysql");
        let url = mysql.database_url().await.expect("mysql url");
        let pool = connect_with_retry(&url).await;

        sqlx::query(include_str!("../ddl/mysql/short_urls.sql"))
            .execute(&pool)
            .await
            .expect("create schema");

        Self {
            _mysql: mysql,
            repo: MySqlRepository::new(pool),
        }
    }
}

async fn connect_with_retry(url: &str) -> sqlx::MySqlPool {
    let mut last_error = None;

    for _ in 0..20 {
        match MySqlPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
        {
            Ok(pool) => return pool,
            Err(err) => {
                last_error = Some(err);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }

    panic!("failed to connect mysql: {last_error:?}");
}

fn code(value: &str) -> ShortCode {
    ShortCode::new_unchecked(value)
}

fn record(url: &str, expire_at: Option<Timestamp>) -> UrlRecord {
    UrlRecord {
        original_url: url.to_string(),
        expire_at,
    }
}

#[tokio::test]
async fn insert_and_get_active_record() {
    let fixture = Fixture::start().await;
    let short_code = code("abc123");

    fixture
        .repo
        .insert(&short_code, record("https://example.com", None))
        .await
        .unwrap();

    let got = fixture.repo.get(&short_code).await.unwrap().unwrap();
    assert_eq!(got.original_url, "https://example.com");
    assert_eq!(got.expire_at, None);
}

#[tokio::test]
async fn insert_conflicts_when_code_already_exists() {
    let fixture = Fixture::start().await;
    let short_code = code("abc123");

    fixture
        .repo
        .insert(&short_code, record("https://one.example", None))
        .await
        .unwrap();

    let err = fixture
        .repo
        .insert(&short_code, record("https://two.example", None))
        .await
        .unwrap_err();

    assert!(matches!(err, StorageError::Conflict(_)));
}

#[tokio::test]
async fn get_returns_none_for_expired_record() {
    let fixture = Fixture::start().await;
    let short_code = code("expired");
    let expired = Timestamp::now() - SignedDuration::from_secs(1);

    fixture
        .repo
        .insert(&short_code, record("https://example.com", Some(expired)))
        .await
        .unwrap();

    let got = fixture.repo.get(&short_code).await.unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn delete_marks_record_as_soft_deleted() {
    let fixture = Fixture::start().await;
    let short_code = code("to-delete");

    fixture
        .repo
        .insert(&short_code, record("https://example.com", None))
        .await
        .unwrap();

    assert!(fixture.repo.delete(&short_code).await.unwrap());
    assert!(fixture.repo.get(&short_code).await.unwrap().is_none());
    assert!(!fixture.repo.delete(&short_code).await.unwrap());
}

#[tokio::test]
async fn exists_tracks_historical_codes_for_no_reuse_policy() {
    let fixture = Fixture::start().await;
    let short_code = code("history");

    fixture
        .repo
        .insert(&short_code, record("https://example.com", None))
        .await
        .unwrap();
    fixture.repo.delete(&short_code).await.unwrap();

    assert!(fixture.repo.exists(&short_code).await.unwrap());
}

use async_trait::async_trait;
use std::sync::Arc;
use typed_builder::TypedBuilder;
use wormhole_core::ShortCode;
use wormhole_redirector::redirector::Redirector;
use wormhole_shortener::shortener::{ExpirationPolicy, ShortenParams, Shortener};

use crate::error::{AppError, Result};
use crate::model::{CreateUrlRequest, CreateUrlResponse, GetUrlResponse};
use crate::port::{UrlReadPort, UrlWritePort};

#[derive(Clone, TypedBuilder)]
pub struct LocalUrlAdapter {
    #[builder(
        setter(
            fn transform<T: Shortener>(shortener: T) -> Arc<dyn Shortener> {
                 Arc::new(shortener)
            }
        )
    )]
    shortener: Arc<dyn Shortener>,
    #[builder(
        setter(
            fn transform<T: Redirector>(redirector: T) -> Arc<dyn Redirector> {
                 Arc::new(redirector)
            }
        )
    )]
    redirector: Arc<dyn Redirector>,
    #[builder(setter(into))]
    base_url: String,
}

impl LocalUrlAdapter {
    fn parse_short_code(short_code: &str) -> Result<ShortCode> {
        ShortCode::custom(short_code).map_err(|error| AppError::InvalidShortCode(error.to_string()))
    }
}

#[async_trait]
impl UrlWritePort for LocalUrlAdapter {
    async fn create(&self, request: CreateUrlRequest) -> Result<CreateUrlResponse> {
        let CreateUrlRequest {
            original_url,
            custom_alias,
            expire_at,
        } = request;

        let custom_alias = custom_alias
            .map(ShortCode::custom)
            .transpose()
            .map_err(|error| AppError::InvalidShortCode(error.to_string()))?;

        let expiration = match expire_at {
            Some(expire_at) => ExpirationPolicy::AtTimestamp(expire_at),
            None => ExpirationPolicy::Never,
        };

        let short_code = self
            .shortener
            .shorten(ShortenParams {
                original_url: original_url.clone(),
                expiration,
                custom_alias,
            })
            .await
            .map_err(AppError::from)?;

        Ok(CreateUrlResponse {
            short_code: short_code.to_string(),
            short_url: short_code.to_url(&self.base_url),
            original_url,
            expire_at,
        })
    }

    async fn delete(&self, short_code: &str) -> Result<()> {
        let short_code = Self::parse_short_code(short_code)?;
        let deleted = self
            .shortener
            .delete(&short_code)
            .await
            .map_err(AppError::from)?;

        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }
}

#[async_trait]
impl UrlReadPort for LocalUrlAdapter {
    async fn get(&self, short_code: &str) -> Result<GetUrlResponse> {
        let short_code = Self::parse_short_code(short_code)?;

        let record = self
            .redirector
            .resolve(&short_code)
            .await
            .map_err(AppError::from)?
            .ok_or(AppError::NotFound)?;

        Ok(GetUrlResponse {
            original_url: record.original_url,
            expire_at: record.expire_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::model::CreateUrlRequest;
    use crate::port::{UrlReadPort, UrlWritePort};
    use wormhole_generator::seq::SeqGenerator;
    use wormhole_redirector::RedirectorService;
    use wormhole_shortener::service::ShortenerService;
    use wormhole_storage::InMemoryRepository;

    #[tokio::test]
    async fn smoke_test() {
        let storge = InMemoryRepository::new();
        let generator = SeqGenerator::with_prefix("test");

        let shortener = ShortenerService::new(storge.clone(), generator);
        let redirector = RedirectorService::new(storge);

        let adapter = super::LocalUrlAdapter::builder()
            .shortener(shortener)
            .redirector(redirector)
            .base_url("https://worm.hole")
            .build();

        // put a url
        let create_response = adapter
            .create(CreateUrlRequest {
                original_url: "https://example.com".to_string(),
                custom_alias: None,
                expire_at: None,
            })
            .await
            .unwrap();

        let code = create_response.short_code;

        // get the url back
        let get_response = adapter.get(&code).await.unwrap();

        assert_eq!(get_response.original_url, "https://example.com");

        // delete the url
        adapter.delete(&code).await.unwrap();

        // try to get the url again, should be not found
        let get_response = adapter.get(&code).await;
        assert!(get_response.is_err());
    }
}

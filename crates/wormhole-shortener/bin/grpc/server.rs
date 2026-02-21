use tonic::{Request, Response, Status};
use wormhole_core::{ShortCode, UrlRecord};
use wormhole_generator::Generator;
use wormhole_proto_schema::v1 as proto;
use wormhole_proto_schema::v1::shortener_service_server::ShortenerService;
use wormhole_proto_schema::v1::{ShortCode as ProtoShortCode, ShortCodeKind};
use wormhole_storage::Repository;

pub struct ShortenerGrpcServer<R: Repository, G: Generator> {
    storage: R,
    generator: G,
}

impl<R: Repository, G: Generator> ShortenerGrpcServer<R, G> {
    pub fn new(storage: R, generator: G) -> Self {
        Self { storage, generator }
    }
}

#[tonic::async_trait]
impl<R: Repository, G: Generator> ShortenerService for ShortenerGrpcServer<R, G> {
    async fn create(
        &self,
        request: Request<proto::CreateRequest>,
    ) -> Result<Response<proto::CreateResponse>, Status> {
        let req = request.into_inner();

        // Validate the URL
        let original_url = req.original_url;
        if original_url.is_empty() {
            return Err(Status::invalid_argument("URL cannot be empty"));
        }

        // Check for valid scheme
        let parts: Vec<&str> = original_url.split("://").collect();
        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(Status::invalid_argument(
                "URL must have a valid scheme and host",
            ));
        }
        let scheme = parts[0].to_lowercase();
        if scheme != "http" && scheme != "https" {
            return Err(Status::invalid_argument("URL scheme must be http or https"));
        }

        // Convert optional expiration timestamp
        let expire_at = req
            .expire_at
            .map(|ts| {
                let seconds = ts.seconds;
                let nanos = ts.nanos;
                jiff::Timestamp::new(seconds, nanos)
                    .map_err(|_| Status::invalid_argument("invalid expiration timestamp"))
            })
            .transpose()?;

        // Determine the short code to use
        let short_code = match req.custom_alias {
            Some(alias) => {
                // Validate and create custom alias
                let code = ShortCode::new(&alias).map_err(|e| {
                    Status::invalid_argument(format!("invalid custom alias: {}", e))
                })?;

                // Check for alias conflict
                if self.storage.exists(&code).await.map_err(Status::from)? {
                    return Err(Status::already_exists("custom alias already exists"));
                }

                code
            }
            None => {
                // Generate new short code
                self.generator.generate().into()
            }
        };

        // Create the URL record
        let record = UrlRecord {
            original_url,
            expire_at,
        };

        // Store in repository
        self.storage
            .insert(&short_code, record)
            .await
            .map_err(Status::from)?;

        // Build response
        let kind = match &short_code {
            ShortCode::Generated(_) => ShortCodeKind::Generated,
            ShortCode::Custom(_) => ShortCodeKind::Custom,
        };

        let response = proto::CreateResponse {
            short_code: Some(ProtoShortCode {
                code: short_code.to_string(),
                kind: kind as i32,
            }),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use crate::server::ShortenerGrpcServer;
    use prost_types::Timestamp;
    use tonic::Request;
    use wormhole_generator::seq::SeqGenerator;
    use wormhole_proto_schema::v1 as proto;
    use wormhole_proto_schema::v1::shortener_service_server::ShortenerService;
    use wormhole_proto_schema::v1::ShortCodeKind;
    use wormhole_storage::InMemoryRepository;

    type TestServer = ShortenerGrpcServer<InMemoryRepository, SeqGenerator>;

    fn test_server() -> TestServer {
        let repo = InMemoryRepository::new();
        let generator = SeqGenerator::with_prefix("test");
        ShortenerGrpcServer::new(repo, generator)
    }

    fn create_request(
        original_url: impl Into<String>,
        expire_at: Option<Timestamp>,
        custom_alias: Option<String>,
    ) -> proto::CreateRequest {
        proto::CreateRequest {
            original_url: original_url.into(),
            expire_at,
            custom_alias,
        }
    }

    #[tokio::test]
    async fn create_with_custom_alias() {
        let server = test_server();

        let request = Request::new(create_request(
            "https://example.com",
            None,
            Some("my-alias".to_string()),
        ));
        let response = server.create(request).await.unwrap();

        let resp = response.into_inner();
        let short_code = resp.short_code.unwrap();

        assert_eq!(short_code.code, "my-alias");
        assert_eq!(short_code.kind, ShortCodeKind::Custom as i32);
    }

    #[tokio::test]
    async fn create_with_duplicate_alias_fails() {
        let server = test_server();

        // First request with custom alias should succeed
        let request1 = Request::new(create_request(
            "https://example1.com",
            None,
            Some("my-alias".to_string()),
        ));
        server.create(request1).await.unwrap();

        // Second request with same alias should fail
        let request2 = Request::new(create_request(
            "https://example2.com",
            None,
            Some("my-alias".to_string()),
        ));
        let result = server.create(request2).await;

        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::AlreadyExists);
    }
}

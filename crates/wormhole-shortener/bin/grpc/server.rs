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

use jiff::Timestamp;
use std::sync::Arc;
use tonic::{Code, Request, Response, Status};
use wormhole_core::{ShortCode, UrlRecord};
use wormhole_proto_schema::v1 as proto;
use wormhole_proto_schema::v1::shortener_service_server::ShortenerService;
use wormhole_shortener::generator::Generator;
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
        todo!()
    }
}

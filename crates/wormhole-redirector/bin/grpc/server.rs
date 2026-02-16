use std::sync::Arc;
use tonic::{Code, Request, Response, Status};
use wormhole_core::{ReadRepository, Repository, ShortCode, UrlCache};
use wormhole_proto_schema::v1::{
    redirector_service_server::RedirectorService, ResolveRequest, ResolveResponse,
};
use wormhole_redirector::CachedRepository;

pub struct RedirectorGrpcServer<R: Repository, C: UrlCache> {
    storage: CachedRepository<R, C>,
}

#[tonic::async_trait]
impl<R: Repository, C: UrlCache> RedirectorService for RedirectorGrpcServer<R, C> {
    async fn resolve(
        &self,
        request: Request<ResolveRequest>,
    ) -> Result<Response<ResolveResponse>, Status> {
        let req = request.into_inner();

        let shortcode: ShortCode = req
            .short_code
            // always require a shortcode
            .ok_or(Status::new(Code::InvalidArgument, "short code is required"))?
            .try_into()
            .map_err(|e| {
                let mut status = Status::new(Code::Internal, "invalid short code");
                status.set_source(Arc::new(e));
                status
            })?;

        let record = self
            .storage
            .get(&shortcode)
            .await
            .expect("storage error") // TODO: map to appropriate gRPC status code
            .ok_or(Status::new(Code::NotFound, "short code not found"))?;

        todo!()
    }
}

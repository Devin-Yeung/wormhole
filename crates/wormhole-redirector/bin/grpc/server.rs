use tonic::{Request, Response, Status};
use wormhole_core::{Repository, UrlCache};
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
        _request: Request<ResolveRequest>,
    ) -> Result<Response<ResolveResponse>, Status> {
        todo!()
    }
}

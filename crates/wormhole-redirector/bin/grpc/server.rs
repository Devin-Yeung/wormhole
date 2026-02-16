use redirector_proto::redirector_service_server::RedirectorService;
use tonic::{Request, Response, Status};
use wormhole_proto_schema::redirector::v1 as redirector_proto;
use wormhole_proto_schema::redirector::v1::{ResolveRequest, ResolveResponse};

pub struct RedirectorGrpcServer {}

#[tonic::async_trait]
impl RedirectorService for RedirectorGrpcServer {
    async fn resolve(
        &self,
        request: Request<ResolveRequest>,
    ) -> Result<Response<ResolveResponse>, Status> {
        todo!()
    }
}

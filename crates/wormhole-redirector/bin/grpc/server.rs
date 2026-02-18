use std::sync::Arc;
use tonic::{Code, Request, Response, Status};
use wormhole_core::{ReadRepository, Repository, ShortCode, StorageError, UrlCache};
use wormhole_proto_schema::v1::{
    redirector_service_server::RedirectorService, ResolveRequest, ResolveResponse, UrlRecord,
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
                let mut status = Status::new(Code::InvalidArgument, "short code is malformed");
                status.set_source(Arc::new(e));
                status
            })?;

        let record = self
            .storage
            .get(&shortcode)
            .await
            .map_err(storage_error_to_status)?
            .ok_or(Status::new(Code::NotFound, "short code not found"))?;

        let wormhole_core::UrlRecord {
            original_url,
            expire_at,
        } = record;

        // We keep this guard at the API boundary so stale cached entries cannot
        // leak expired records through gRPC responses.
        let expire_at = match expire_at {
            Some(expire_at) if jiff::Timestamp::now() >= expire_at => {
                return Err(Status::new(Code::NotFound, "short code not found"));
            }
            Some(expire_at) => {
                let mut ts = prost_types::Timestamp::default();
                ts.seconds = expire_at.as_second();
                Some(ts)
            }
            None => None,
        };

        Ok(Response::new(ResolveResponse {
            url_record: Some(UrlRecord {
                original_url,
                expire_at,
            }),
        }))
    }
}

fn storage_error_to_status(error: StorageError) -> Status {
    let (code, message) = match &error {
        StorageError::Unavailable(_) | StorageError::Cache(_) => {
            (Code::Unavailable, "backend is unavailable")
        }
        StorageError::Timeout(_) => (Code::DeadlineExceeded, "backend timed out"),
        StorageError::Conflict(_) | StorageError::Query(_) | StorageError::InvalidData(_) => {
            (Code::Internal, "backend operation failed")
        }
        StorageError::Operation(_) => (Code::Internal, "backend operation failed"),
    };

    let mut status = Status::new(code, message);
    status.set_source(Arc::new(error));
    status
}

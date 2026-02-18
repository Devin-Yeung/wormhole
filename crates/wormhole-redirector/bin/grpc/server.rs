use proto::redirector_service_server::RedirectorService;
use tonic::{Request, Response, Status};
use wormhole_core::{ReadRepository, Repository, ShortCode, UrlCache, UrlRecord};
use wormhole_proto_schema::v1 as proto;

use wormhole_redirector::CachedRepository;

use crate::error::RedirectorError;

pub struct RedirectorGrpcServer<R: Repository, C: UrlCache> {
    storage: CachedRepository<R, C>,
}

struct ResolveRequest {
    short_code: ShortCode,
}

impl TryFrom<proto::ResolveRequest> for ResolveRequest {
    type Error = RedirectorError;

    fn try_from(value: proto::ResolveRequest) -> Result<Self, Self::Error> {
        let shortcode = value
            .short_code
            // always require a shortcode
            .ok_or(RedirectorError::ShortCodeRequired)?;
        let shortcode: ShortCode = shortcode.try_into()?;

        let req = ResolveRequest {
            short_code: shortcode,
        };

        Ok(req)
    }
}

struct ResolveResponse {
    url_record: UrlRecord,
}

impl TryInto<proto::ResolveResponse> for ResolveResponse {
    type Error = RedirectorError;

    fn try_into(self) -> Result<proto::ResolveResponse, Self::Error> {
        let UrlRecord {
            original_url,
            expire_at,
        } = self.url_record;

        // We keep this guard at the API boundary so stale cached entries cannot
        // leak expired records through gRPC responses.
        let expire_at = match expire_at {
            Some(expire_at) if jiff::Timestamp::now() >= expire_at => {
                return Err(RedirectorError::ShortCodeNotFound);
            }
            Some(expire_at) => {
                let mut ts = prost_types::Timestamp::default();
                ts.seconds = expire_at.as_second();
                Some(ts)
            }
            None => None,
        };

        Ok(proto::ResolveResponse {
            url_record: Some(proto::UrlRecord {
                original_url,
                expire_at,
            }),
        })
    }
}

#[tonic::async_trait]
impl<R: Repository, C: UrlCache> RedirectorService for RedirectorGrpcServer<R, C> {
    async fn resolve(
        &self,
        request: Request<proto::ResolveRequest>,
    ) -> Result<Response<proto::ResolveResponse>, Status> {
        let req: ResolveRequest = request.into_inner().try_into()?;

        let record = self
            .storage
            .get(&req.short_code)
            .await
            .map_err(RedirectorError::from)?
            .ok_or(RedirectorError::ShortCodeNotFound)?;

        let resp: proto::ResolveResponse = ResolveResponse { url_record: record }.try_into()?;

        Ok(Response::new(resp))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use jiff::{SignedDuration, Timestamp};
    use tonic::Code;

    fn resolve_response(expire_at: Option<Timestamp>) -> ResolveResponse {
        ResolveResponse {
            url_record: UrlRecord {
                original_url: "https://example.com".to_string(),
                expire_at,
            },
        }
    }

    #[test]
    fn resolve_response_try_into_converts_non_expiring_record() {
        let response: proto::ResolveResponse = resolve_response(None)
            .try_into()
            .expect("response should convert");

        let record = response.url_record.expect("record should be present");
        assert_eq!(record.original_url, "https://example.com");
        assert!(record.expire_at.is_none());
    }

    #[test]
    fn resolve_response_try_into_converts_future_expiration() {
        let expire_at = Timestamp::now() + SignedDuration::from_secs(60);

        let response: proto::ResolveResponse = resolve_response(Some(expire_at))
            .try_into()
            .expect("response should convert");

        let record = response.url_record.expect("record should be present");
        assert_eq!(record.original_url, "https://example.com");

        let proto_expire_at = record.expire_at.expect("expiration should be present");
        assert_eq!(proto_expire_at.seconds, expire_at.as_second());
    }

    #[test]
    fn resolve_response_try_into_rejects_expired_records() {
        let expire_at = Timestamp::now() - SignedDuration::from_secs(1);

        let result: Result<proto::ResolveResponse, RedirectorError> =
            resolve_response(Some(expire_at)).try_into();
        let status: Status = result
            .expect_err("expired record should be rejected")
            .into();

        assert_eq!(status.code(), Code::NotFound);
        assert_eq!(status.message(), "short code not found");
    }
}

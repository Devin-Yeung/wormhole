//! gRPC client adapter for the gateway.
//!
//! This adapter connects to remote shortener and redirector microservices via gRPC,
//! implementing the `UrlWrite` and `UrlRead` traits to provide a unified interface
//! for the gateway. This keeps the gateway decoupled from the underlying communication
//! protocol - it only knows about the trait methods, not how they're implemented.

use async_trait::async_trait;
use tonic::transport::Channel;
use typed_builder::TypedBuilder;
use wormhole_proto_schema::v1 as proto;
use wormhole_proto_schema::v1::redirector_service_client::RedirectorServiceClient;
use wormhole_proto_schema::v1::shortener_service_client::ShortenerServiceClient;
use wormhole_proto_schema::v1::{ShortCode, ShortCodeKind};

use crate::backend::{
    BackendError, DeleteUrlCmd, GetUrlResult, Result, UrlRead, UrlWrite, WriteUrlCmd,
    WriteUrlResult,
};

// ==============================================================================
// GrpcUrlAdapter
// ==============================================================================

/// gRPC client adapter that connects to remote shortener and redirector services.
///
/// This adapter acts as a client to the microservices, translating calls to
/// `UrlWrite` and `UrlRead` into gRPC requests to the respective services:
/// - `UrlWrite` operations (create, delete) → ShortenerService
/// - `UrlRead` operations (get, resolve) → RedirectorService
#[derive(TypedBuilder)]
pub struct GrpcUrlAdapter {
    /// gRPC client for the shortener service (write operations).
    #[builder]
    shortener: ShortenerServiceClient<Channel>,
    /// gRPC client for the redirector service (read operations).
    #[builder]
    redirector: RedirectorServiceClient<Channel>,
}

impl GrpcUrlAdapter {
    /// Creates a new adapter with the given gRPC clients.
    pub fn new(
        shortener: ShortenerServiceClient<Channel>,
        redirector: RedirectorServiceClient<Channel>,
    ) -> Self {
        Self {
            shortener,
            redirector,
        }
    }
}

// ==============================================================================
// UrlWrite Implementation
// ==============================================================================

#[async_trait]
impl UrlWrite for GrpcUrlAdapter {
    async fn create(&self, cmd: WriteUrlCmd) -> Result<WriteUrlResult> {
        // Convert expiration to protobuf timestamp if present
        let expire_at = cmd.expire_at.map(|ts| prost_types::Timestamp {
            seconds: ts.as_second(),
            nanos: ts.as_nanosecond() as i32,
        });

        // Build the gRPC request
        let original_url = cmd.original_url.clone();
        let request = proto::CreateRequest {
            original_url: cmd.original_url,
            custom_alias: cmd.custom_alias,
            expire_at,
        };

        // Call the remote shortener service
        let response = self
            .shortener
            .clone()
            .create(request)
            .await
            .map_err(|e| BackendError::Internal(e.to_string()))?
            .into_inner();

        // Extract the short code from response
        let short_code = response
            .short_code
            .ok_or_else(|| BackendError::Internal("missing short_code in response".to_string()))?;

        let short_code_str = short_code.code.clone();

        // Note: The gRPC response doesn't include short_url or original_url
        // We reconstruct what we can from the request
        Ok(WriteUrlResult {
            short_code: short_code_str.clone(),
            short_url: format!("https://worm.hole/{}", short_code_str),
            original_url,
            expire_at: cmd.expire_at,
        })
    }

    async fn delete(&self, cmd: DeleteUrlCmd) -> Result<()> {
        // Build the short code for the request
        let _short_code = ShortCode {
            code: cmd.short_code,
            kind: ShortCodeKind::Generated as i32,
        };

        // Call the remote shortener service's delete method
        // Note: The proto only defines Create, so we'd need to add Delete to the proto
        // For now, we'll handle this as a not-implemented error
        Err(BackendError::Internal(
            "delete not implemented in gRPC adapter (requires proto extension)".to_string(),
        ))
    }
}

// ==============================================================================
// UrlRead Implementation
// ==============================================================================

#[async_trait]
impl UrlRead for GrpcUrlAdapter {
    async fn get(&self, short_code: &str) -> Result<GetUrlResult> {
        // Build the short code for the request
        let short_code = ShortCode {
            code: short_code.to_string(),
            kind: ShortCodeKind::Generated as i32,
        };

        let request = proto::ResolveRequest {
            short_code: Some(short_code),
        };

        // Call the remote redirector service
        let response = self
            .redirector
            .clone()
            .resolve(request)
            .await
            .map_err(|e| BackendError::Internal(e.to_string()))?
            .into_inner();

        // Extract the URL record from response
        let url_record = response.url_record.ok_or_else(|| BackendError::NotFound)?;

        // Convert expiration timestamp if present
        let expire_at = url_record
            .expire_at
            .map(|ts| jiff::Timestamp::new(ts.seconds, ts.nanos as i32).expect("valid timestamp"));

        Ok(GetUrlResult {
            original_url: url_record.original_url,
            expire_at,
        })
    }
}

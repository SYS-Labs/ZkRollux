pub mod types;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use hex::encode as hex_encode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DAError {
    #[error("Network error: {0}")]
    NetworkError(reqwest::Error),
    #[error("Serialization error: {0}")]
    SerializationError(serde_json::Error),
}

type Result<T> = std::result::Result<T, DAError>;

#[derive(Serialize, Deserialize, Debug)]
struct DispatchResponse {
    vh: String, // Version hash returned by Syscoin RPC
}

#[async_trait]
pub trait DataAvailabilityClient: Sync + Send + std::fmt::Debug {
    async fn dispatch_blob(&self, data: Vec<u8>) -> Result<DispatchResponse>;
    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient>;
}

#[derive(Clone, Debug)]
struct SyscoinClient {
    client: Client,
    rpc_url: String,
    user: String,
    password: String,
}

#[async_trait]
impl DataAvailabilityClient for SyscoinClient {
    async fn dispatch_blob(&self, data: Vec<u8>) -> Result<DispatchResponse> {
        let endpoint = format!("{}/", self.rpc_url);
        let params = serde_json::json!({
            "method": "syscoincreatenevmblob",
            "params": {
                "data": hex_encode(data)
            }
        });

        let client = self.client.clone();
        let response = client.post(endpoint)
            .basic_auth(&self.user, Some(&self.password))
            .json(&params)
            .send()
            .await
            .map_err(DAError::NetworkError)?;

        let result = response
            .json::<RpcResponse>()
            .await
            .map_err(DAError::SerializationError)?;

        if let Some(error) = result.error {
            Err(DAError::NetworkError(reqwest::Error::new(
                reqwest::ErrorKind::Other,
                format!("RPC error: {}", error.message),
            )))
        } else {
            Ok(DispatchResponse { vh: result.result.vh })
        }
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> Option<usize> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcResponse {
    result: DispatchResult,
    error: Option<RpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DispatchResult {
    vh: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcError {
    code: i32,
    message: String,
}

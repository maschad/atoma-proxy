use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

/// A request for confidential computation that includes encrypted data and associated cryptographic parameters
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ConfidentialComputeRequest {
    /// The encrypted payload that needs to be processed (base64 encoded)
    pub ciphertext: String,

    /// Unique identifier for the small stack being used
    pub stack_small_id: u64,

    /// Cryptographic nonce used for encryption (base64 encoded)
    pub nonce: String,

    /// Salt value used in key derivation (base64 encoded)
    pub salt: String,

    /// Client's public key for Diffie-Hellman key exchange (base64 encoded)
    pub client_dh_public_key: String,

    /// Node's public key for Diffie-Hellman key exchange (base64 encoded)
    pub node_dh_public_key: String,

    /// Hash of the original plaintext body for integrity verification (base64 encoded)
    pub plaintext_body_hash: String,

    /// Indicates whether this is a streaming request
    pub stream: Option<bool>,

    /// Model name
    pub model_name: String,

    /// Number of compute units to be used for the request, for image generations,
    /// as this value is known in advance (the number of pixels to generate)
    pub num_compute_units: Option<u64>,
}

/// Represents a response from a confidential compute request
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ConfidentialComputeResponse {
    /// Encrypted response body (base64 encoded)
    pub ciphertext: String,

    /// Nonce used for encryption (base64 encoded)
    pub nonce: String,

    /// Signature of the response body (base64 encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// Hash of the response body (base64 encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_hash: Option<String>,

    /// Usage statistics for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Represents a response from a confidential compute request
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ConfidentialComputeStreamResponse {
    /// The stream of chat completion chunks.
    pub data: ConfidentialComputeResponse,
}

/// Represents usage statistics for a confidential compute request
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Usage {
    /// Number of compute units used
    pub prompt_tokens: u64,

    /// Number of compute units used
    /// NOTE: This is not used for the embeddings endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u64>,

    /// Number of compute units used
    pub total_tokens: u64,

    /// Details about the completion tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<Value>,
}

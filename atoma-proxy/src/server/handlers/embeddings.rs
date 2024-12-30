use std::time::Instant;

use atoma_state::types::AtomaAtomaStateManagerEvent;
use atoma_utils::constants;
use axum::{
    body::Body,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::chrono::{DateTime, Utc};
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};
use x25519_dalek::PublicKey;

use crate::server::{
    error::AtomaProxyError,
    handlers::{extract_node_encryption_metadata, handle_confidential_compute_decryption_response},
    http_server::ProxyState,
    middleware::{NodeEncryptionMetadata, RequestMetadataExtension},
};

use super::request_model::RequestModel;
use crate::server::Result;

/// Path for the confidential embeddings endpoint.
///
/// This endpoint follows the OpenAI API format for embeddings, with additional
/// confidential processing (through AEAD encryption and TEE hardware).
pub const CONFIDENTIAL_EMBEDDINGS_PATH: &str = "/v1/confidential/embeddings";

/// Path for the embeddings endpoint.
///
/// This endpoint follows the OpenAI API format for embeddings
/// and is used to generate vector embeddings for input text.
pub const EMBEDDINGS_PATH: &str = "/v1/embeddings";

/// The model field in the request payload.
const MODEL: &str = "model";

/// The input field in the request payload.
const INPUT: &str = "input";

// A model representing an embeddings request payload.
///
/// This struct encapsulates the necessary fields for processing an embeddings request
/// following the OpenAI API format.
pub struct RequestModelEmbeddings {
    /// The name of the model to use for generating embeddings (e.g., "text-embedding-ada-002")
    model: String,
    /// The input text to generate embeddings for
    input: String,
}

/// OpenAPI documentation for the embeddings endpoint.
#[derive(OpenApi)]
#[openapi(
    paths(embeddings_create),
    components(schemas(
        CreateEmbeddingRequest,
        EmbeddingObject,
        EmbeddingUsage,
        CreateEmbeddingResponse
    ))
)]
pub(crate) struct EmbeddingsOpenApi;

impl RequestModel for RequestModelEmbeddings {
    fn new(request: &Value) -> Result<Self> {
        let model =
            request
                .get(MODEL)
                .and_then(|m| m.as_str())
                .ok_or(AtomaProxyError::InvalidBody {
                    message: "Model field is required".to_string(),
                    endpoint: EMBEDDINGS_PATH.to_string(),
                })?;
        let input =
            request
                .get(INPUT)
                .and_then(|i| i.as_str())
                .ok_or(AtomaProxyError::InvalidBody {
                    message: "Input field is required".to_string(),
                    endpoint: EMBEDDINGS_PATH.to_string(),
                })?;

        Ok(Self {
            model: model.to_string(),
            input: input.to_string(),
        })
    }

    fn get_model(&self) -> Result<String> {
        Ok(self.model.clone())
    }

    fn get_compute_units_estimate(&self, state: &ProxyState) -> Result<u64> {
        let tokenizer_index = state
            .models
            .iter()
            .position(|m| m == &self.model)
            .ok_or_else(|| AtomaProxyError::InvalidBody {
                message: "Model not supported".to_string(),
                endpoint: EMBEDDINGS_PATH.to_string(),
            })?;
        let tokenizer = &state.tokenizers[tokenizer_index];

        let num_tokens = tokenizer
            .encode(self.input.as_str(), true)
            .map_err(|err| AtomaProxyError::InternalError {
                message: format!("Failed to encode input: {:?}", err),
                endpoint: EMBEDDINGS_PATH.to_string(),
            })?
            .get_ids()
            .len() as u64;

        Ok(num_tokens)
    }
}

/// Create embeddings
///
/// This endpoint follows the OpenAI API format for generating vector embeddings from input text.
/// The handler receives pre-processed metadata from middleware and forwards the request to
/// the selected node.
///
/// Note: Authentication, node selection, and initial request validation are handled by middleware
/// before this handler is called.
///
/// # Arguments
/// * `metadata` - Pre-processed request metadata containing node information and compute units
/// * `state` - The shared proxy state containing configuration and runtime information
/// * `headers` - HTTP headers from the incoming request
/// * `payload` - The JSON request body containing the model and input text
///
/// # Returns
/// * `Ok(Response)` - The embeddings response from the processing node
/// * `Err(AtomaProxyError)` - An error status code if any step fails
///
/// # Errors
/// * `INTERNAL_SERVER_ERROR` - Processing or node communication failures
#[utoipa::path(
    post,
    path = "",
    request_body = CreateEmbeddingRequest,
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = OK, description = "Embeddings generated successfully", body = CreateEmbeddingResponse),
        (status = BAD_REQUEST, description = "Bad request"),
        (status = UNAUTHORIZED, description = "Unauthorized"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error")
    )
)]
#[instrument(
    level = "info",
    skip_all,
    fields(endpoint = metadata.endpoint)
)]
pub async fn embeddings_create(
    Extension(metadata): Extension<RequestMetadataExtension>,
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response<Body>> {
    let RequestMetadataExtension {
        node_address,
        node_id,
        num_compute_units: num_input_compute_units,
        ..
    } = metadata;
    handle_embeddings_response(
        state,
        node_address,
        node_id,
        headers,
        payload,
        num_input_compute_units as i64,
        metadata.endpoint,
        metadata.salt,
        metadata.node_x25519_public_key,
        metadata.model_name,
    )
    .await
}

/// Atoma's confidential embeddings OpenAPI documentation.
#[derive(OpenApi)]
#[openapi(
    paths(confidential_embeddings_create),
    components(schemas(
        CreateEmbeddingRequest,
        EmbeddingObject,
        EmbeddingUsage,
        CreateEmbeddingResponse
    ))
)]
pub(crate) struct ConfidentialEmbeddingsOpenApi;

/// Create confidential embeddings
///
/// This endpoint follows the OpenAI API format for generating vector embeddings from input text,
/// but with confidential processing (through AEAD encryption and TEE hardware).
/// The handler receives pre-processed metadata from middleware and forwards the request to
/// the selected node.
///
/// Note: Authentication, node selection, initial request validation and encryption
/// are handled by middleware before this handler is called.
///
/// # Arguments
/// * `metadata` - Pre-processed request metadata containing node information and compute units
/// * `state` - The shared proxy state containing configuration and runtime information
/// * `headers` - HTTP headers from the incoming request
/// * `payload` - The JSON request body containing the model and input text
///
/// # Returns
/// * `Ok(Response)` - The embeddings response from the processing node
/// * `Err(AtomaProxyError)` - An error status code if any step fails
///
/// # Errors
/// * `INTERNAL_SERVER_ERROR` - Processing or node communication failures
#[utoipa::path(
    post,
    path = "",
    request_body = CreateEmbeddingRequest,
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = OK, description = "Confidential embeddings generated successfully", body = CreateEmbeddingResponse),
        (status = BAD_REQUEST, description = "Bad request"),
        (status = UNAUTHORIZED, description = "Unauthorized"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error")
    )
)]
#[instrument(
    level = "info",
    skip_all,
    fields(endpoint = metadata.endpoint)
)]
pub async fn confidential_embeddings_create(
    Extension(metadata): Extension<RequestMetadataExtension>,
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response<Body>> {
    let RequestMetadataExtension {
        node_address,
        node_id,
        num_compute_units: num_input_compute_units,
        ..
    } = metadata;
    handle_embeddings_response(
        state,
        node_address,
        node_id,
        headers,
        payload,
        num_input_compute_units as i64,
        metadata.endpoint,
        metadata.salt,
        metadata.node_x25519_public_key,
        metadata.model_name,
    )
    .await
}

/// Handles the response processing for embeddings requests by forwarding them to AI nodes and managing performance metrics.
///
/// This function is responsible for:
/// 1. Forwarding the embeddings request to the selected AI node
/// 2. Processing the node's response
/// 3. Updating performance metrics for the node
///
/// # Arguments
/// * `state` - The shared proxy state containing configuration and runtime information
/// * `node_address` - The URL of the selected AI node
/// * `selected_node_id` - The unique identifier of the selected node
/// * `signature` - Authentication signature for the node request
/// * `selected_stack_small_id` - The identifier for the selected processing stack
/// * `headers` - HTTP headers to forward with the request
/// * `payload` - The JSON request body containing the embeddings request
/// * `num_input_compute_units` - The number of compute units (tokens) in the input
///
/// # Returns
/// * `Ok(Response<Body>)` - The processed embeddings response from the AI node
/// * `Err(AtomaProxyError)` - An error status code if any step fails
///
/// # Errors
/// * Returns `INTERNAL_SERVER_ERROR` if:
///   - The request to the AI node fails
///   - The response parsing fails
///   - Updating node performance metrics fails
#[instrument(
    level = "info",
    skip_all,
    fields(
        path = endpoint,
        stack_small_id,
        estimated_total_tokens
    )
)]
#[allow(clippy::too_many_arguments)]
async fn handle_embeddings_response(
    state: ProxyState,
    node_address: String,
    selected_node_id: i64,
    headers: HeaderMap,
    payload: Value,
    num_input_compute_units: i64,
    endpoint: String,
    salt: Option<[u8; constants::SALT_SIZE]>,
    node_x25519_public_key: Option<PublicKey>,
    model_name: String,
) -> Result<Response<Body>> {
    let client = reqwest::Client::new();
    let time = Instant::now();
    // Send the request to the AI node
    let response = client
        .post(format!("{}{}", node_address, endpoint))
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|err| AtomaProxyError::InternalError {
            message: format!("Failed to send embeddings request: {:?}", err),
            endpoint: endpoint.to_string(),
        })?
        .json::<Value>()
        .await
        .map_err(|err| AtomaProxyError::InternalError {
            message: format!("Failed to parse embeddings response: {:?}", err),
            endpoint: endpoint.to_string(),
        })
        .map(Json)?;

    let response = if let (Some(node_x25519_public_key), Some(salt)) =
        (node_x25519_public_key, salt)
    {
        let shared_secret = state.compute_shared_secret(&node_x25519_public_key);
        let NodeEncryptionMetadata { ciphertext, nonce } =
            extract_node_encryption_metadata(response.0)?;
        handle_confidential_compute_decryption_response(shared_secret, &ciphertext, &salt, &nonce)?
    } else {
        response.0
    };

    // Update the node throughput performance
    state
        .state_manager_sender
        .send(
            AtomaAtomaStateManagerEvent::UpdateNodeThroughputPerformance {
                timestamp: DateTime::<Utc>::from(std::time::SystemTime::now()),
                model_name,
                node_small_id: selected_node_id,
                input_tokens: num_input_compute_units,
                output_tokens: 0,
                time: time.elapsed().as_secs_f64(),
            },
        )
        .map_err(|err| AtomaProxyError::InternalError {
            message: format!("Failed to update node throughput performance: {:?}", err),
            endpoint: endpoint.to_string(),
        })?;

    Ok(Json(response).into_response())
}

/// Request object for creating embeddings
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateEmbeddingRequest {
    /// ID of the model to use.
    pub model: String,

    /// Input text to get embeddings for. Can be a string or array of strings.
    /// Each input must not exceed the max input tokens for the model
    #[serde(flatten)]
    pub input: EmbeddingInput,

    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// The format to return the embeddings in. Can be "float" or "base64".
    /// Defaults to "float"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,

    /// The number of dimensions the resulting output embeddings should have.
    /// Only supported in text-embedding-3 models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
}

/// Input types for embeddings request
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum EmbeddingInput {
    Single(String),
    Multiple(Vec<String>),
}

/// Response object from creating embeddings
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateEmbeddingResponse {
    /// The object type, which is always "list"
    pub object: String,

    /// The model used for generating embeddings
    pub model: String,

    /// List of embedding objects
    pub data: Vec<EmbeddingObject>,

    /// Usage statistics for the request
    pub usage: EmbeddingUsage,
}

/// Individual embedding object in the response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EmbeddingObject {
    /// The object type, which is always "embedding"
    pub object: String,

    /// The embedding vector
    pub embedding: Vec<f32>,

    /// Index of the embedding in the list of embeddings
    pub index: usize,
}

/// Usage information for the embeddings request
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EmbeddingUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,

    /// Total tokens used in the request
    pub total_tokens: u32,
}

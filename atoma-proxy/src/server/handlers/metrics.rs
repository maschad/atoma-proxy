use once_cell::sync::Lazy;
use opentelemetry::{
    global,
    metrics::{Counter, Histogram, Meter},
};

// Add global metrics
static GLOBAL_METER: Lazy<Meter> = Lazy::new(|| global::meter("atoma-proxy"));

const LATENCY_HISTOGRAM_BUCKETS: [f64; 15] = [
    0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0,
];

/// Counter metric that tracks the total number of chat completion requests.
///
/// This metric counts the number of incoming requests for chat completions,
/// broken down by model type. This helps monitor usage patterns and load
/// across different models.
///
/// # Metric Details
/// - Name: `atoma_chat_completions_num_requests`
/// - Type: Counter
/// - Labels: `model`
/// - Unit: requests (count)
pub static CHAT_COMPLETIONS_NUM_REQUESTS: Lazy<Counter<u64>> = Lazy::new(|| {
    GLOBAL_METER
        .u64_counter("atoma_chat_completions_num_requests")
        .with_description("The number of incoming requests for chat completions tasks")
        .with_unit("s")
        .build()
});

/// Counter metric that tracks the total number of image generation requests.
///
/// This metric counts the number of incoming requests for image generations,
/// broken down by model type. This helps monitor usage patterns and load
/// across different image generation models.
///
/// # Metric Details
/// - Name: `atoma_image_gen_num_requests`
/// - Type: Counter
/// - Labels: `model`
/// - Unit: requests (count)
pub static IMAGE_GEN_NUM_REQUESTS: Lazy<Counter<u64>> = Lazy::new(|| {
    GLOBAL_METER
        .u64_counter("atoma_image_gen_num_requests")
        .with_description("The number of incoming requests for image generation tasks")
        .with_unit("requests")
        .build()
});

/// Counter metric that tracks the total number of text embedding requests.
///
/// This metric counts the number of incoming requests for text embeddings,
/// broken down by model type. This helps monitor usage patterns and load
/// across different embedding models.
///
/// # Metric Details
/// - Name: `atoma_text_embs_num_requests`
/// - Type: Counter
/// - Labels: `model`
/// - Unit: requests (count)
pub static TEXT_EMBEDDINGS_NUM_REQUESTS: Lazy<Counter<u64>> = Lazy::new(|| {
    GLOBAL_METER
        .u64_counter("atoma_text_embs_num_requests")
        .with_description("The number of incoming requests for text embeddings tasks")
        .with_unit("requests")
        .build()
});

/// Histogram metric that tracks the latency of chat completion token generation.
///
/// This metric measures the time taken to generate each token during chat completions,
/// broken down by model type. The histogram buckets range from 10ms to 10 minutes to
/// capture both fast and slow token generation scenarios.
///
/// # Metric Details
/// - Name: `atoma_chat_completions_token_latency`
/// - Type: Histogram
/// - Labels: `model`
/// - Unit: seconds
/// - Buckets: [0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0]
pub static CHAT_COMPLETIONS_LATENCY_METRICS: Lazy<Histogram<f64>> = Lazy::new(|| {
    GLOBAL_METER
        .f64_histogram("atoma_chat_completions_token_latency")
        .with_description("The latency of chat completion generation in seconds")
        .with_unit("s")
        .with_boundaries(LATENCY_HISTOGRAM_BUCKETS.to_vec())
        .build()
});

/// Histogram metric that tracks the latency of chat completion streaming requests.
///
/// This metric measures the time taken to stream chat completions, broken down by model type.
/// The histogram buckets range from 1ms to 10 minutes to capture both fast and slow
/// streaming scenarios.
///
/// # Metric Details
/// - Name: `atoma_chat_completions_streaming_latency`
/// - Type: Histogram
/// - Labels: `model`
/// - Unit: seconds
/// - Buckets: [0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0]
pub static CHAT_COMPLETIONS_STREAMING_LATENCY_METRICS: Lazy<Histogram<f64>> = Lazy::new(|| {
    GLOBAL_METER
        .f64_histogram("atoma_chat_completions_streaming_latency")
        .with_description("The latency of chat completion streaming in seconds")
        .with_unit("s")
        .build()
});

/// Histogram metric that tracks the latency of image generation requests.
///
/// This metric measures the time taken to generate images, broken down by model type.
/// The histogram buckets range from 1ms to 10 minutes to capture both fast and slow
/// generation scenarios.
///
/// # Metric Details
/// - Name: `atoma_image_generation_latency`
/// - Type: Histogram
/// - Labels: `model`
/// - Unit: seconds
/// - Buckets: [0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0]
pub static IMAGE_GEN_LATENCY_METRICS: Lazy<Histogram<f64>> = Lazy::new(|| {
    GLOBAL_METER
        .f64_histogram("atoma_image_generation_latency")
        .with_description("The latency of image generation in seconds")
        .with_unit("s")
        .with_boundaries(LATENCY_HISTOGRAM_BUCKETS.to_vec())
        .build()
});

/// Histogram metric that tracks the latency of text embedding requests.
///
/// This metric measures the time taken to generate text embeddings, broken down by model type.
/// The histogram buckets range from 1ms to 10 minutes to capture both fast and slow
/// embedding generation scenarios.
///
/// # Metric Details
/// - Name: `atoma_text_embeddings_latency`
/// - Type: Histogram
/// - Labels: `model`
/// - Unit: seconds
/// - Buckets: [0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0]
pub static TEXT_EMBEDDINGS_LATENCY_METRICS: Lazy<Histogram<f64>> = Lazy::new(|| {
    GLOBAL_METER
        .f64_histogram("atoma_text_embeddings_latency")
        .with_description("The latency of text embeddings in seconds")
        .with_unit("s")
        .with_boundaries(LATENCY_HISTOGRAM_BUCKETS.to_vec())
        .build()
});

/// Counter metric that tracks the total number of tokens processed in chat completions.
///
/// This metric counts the cumulative number of tokens in the input prompts,
/// broken down by model type. This helps monitor token usage and costs
/// across different models and client applications.
///
/// # Metric Details
/// - Name: `atoma_chat_completions_input_tokens_metrics`
/// - Type: Counter
/// - Labels:
///   - `model`: The model used for completion
/// - Unit: tokens (count)
pub static CHAT_COMPLETIONS_ESTIMATED_TOTAL_TOKENS: Lazy<Counter<u64>> = Lazy::new(|| {
    GLOBAL_METER
        .u64_counter("atoma_chat_completions_estimated_total_tokens")
        .with_description("The estimated total number of tokens processed")
        .with_unit("tokens")
        .build()
});

/// Counter metrics that tracks the total number of successfully completed requests (including chat completions, image generation, and text embeddings)
///
/// # Metric Details
/// - Name: `atoma_total_completed_requests`
/// - Type: Counter
/// - Labels: `model`
/// - Unit: requests (count)
pub static TOTAL_COMPLETED_REQUESTS: Lazy<Counter<u64>> = Lazy::new(|| {
    GLOBAL_METER
        .u64_counter("atoma_total_completed_requests")
        .with_description("Total number of successfully completed requests")
        .with_unit("requests")
        .build()
});

/// Counter metric that tracks the total number of failed requests (including chat completions, image generation, and text embeddings)
///
/// # Metric Details
/// - Name: `atoma_total_failed_requests`
/// - Type: Counter
/// - Labels: `model`
/// - Unit: requests (count)
pub static TOTAL_FAILED_REQUESTS: Lazy<Counter<u64>> = Lazy::new(|| {
    GLOBAL_METER
        .u64_counter("atoma_total_failed_requests")
        .with_description("Total number of failed requests")
        .with_unit("requests")
        .build()
});

/// Counter metric that tracks the total number of failed chat requests.
///
/// # Metric Details
/// - Name: `atoma_total_failed_chat_requests`
/// - Type: Counter
/// - Labels: `model`
/// - Unit: requests (count)
pub static TOTAL_FAILED_CHAT_REQUESTS: Lazy<Counter<u64>> = Lazy::new(|| {
    GLOBAL_METER
        .u64_counter("atoma_total_failed_chat_requests")
        .with_description("Total number of failed chat requests")
        .with_unit("requests")
        .build()
});

/// Counter metric that tracks the total number of failed image generation requests.
///
/// # Metric Details
/// - Name: `atoma_total_failed_image_generation_requests`
/// - Type: Counter
/// - Labels: `model`
/// - Unit: requests (count)
pub static TOTAL_FAILED_IMAGE_GENERATION_REQUESTS: Lazy<Counter<u64>> = Lazy::new(|| {
    GLOBAL_METER
        .u64_counter("atoma_total_failed_image_generation_requests")
        .with_description("Total number of failed image generation requests")
        .with_unit("requests")
        .build()
});

/// Counter metric that tracks the total number of failed text embedding requests.
///
/// # Metric Details
/// - Name: `atoma_total_failed_text_embedding_requests`
/// - Type: Counter
/// - Labels: `model`
/// - Unit: requests (count)
pub static TOTAL_FAILED_TEXT_EMBEDDING_REQUESTS: Lazy<Counter<u64>> = Lazy::new(|| {
    GLOBAL_METER
        .u64_counter("atoma_total_failed_text_embedding_requests")
        .with_description("Total number of failed text embedding requests")
        .with_unit("requests")
        .build()
});

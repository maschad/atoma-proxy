use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::server::handlers::{
    chat_completions::ChatCompletionsOpenApi, chat_completions::CHAT_COMPLETIONS_PATH,
    embeddings::EmbeddingsOpenApi, embeddings::EMBEDDINGS_PATH,
    image_generations::ImageGenerationsOpenApi, image_generations::IMAGE_GENERATIONS_PATH,
};
use crate::server::http_server::{HealthOpenApi, ModelsOpenApi, HEALTH_PATH, MODELS_PATH};

pub fn openapi_routes() -> Router {
    #[derive(OpenApi)]
    #[openapi(
        nest(
            (path = HEALTH_PATH, api = HealthOpenApi, tags = ["Health"]),
            (path = MODELS_PATH, api = ModelsOpenApi, tags = ["Models"]),
            (path = CHAT_COMPLETIONS_PATH, api = ChatCompletionsOpenApi, tags = ["Chat Completions"]),
            (path = EMBEDDINGS_PATH, api = EmbeddingsOpenApi, tags = ["Embeddings"]),
            (path = IMAGE_GENERATIONS_PATH, api = ImageGenerationsOpenApi, tags = ["Image Generations"]),
        ),
        tags(
            (name = "Health", description = "Health check"),
            (name = "Chat Completions", description = "Chat completions"),
            (name = "Models", description = "Models"),
            (name = "Embeddings", description = "OpenAI's API embeddings v1 endpoint"),
            (name = "Image Generations", description = "OpenAI's API image generations v1 endpoint"),
        ),
        servers(
            (url = "http://localhost:8080"),
        )
    )]
    struct ApiDoc;

    // Generate the OpenAPI spec and write it to a file
    #[cfg(debug_assertions)]
    {
        use std::fs;
        use std::path::Path;

        // Generate OpenAPI spec
        let spec =
            serde_yaml::to_string(&ApiDoc::openapi()).expect("Failed to serialize OpenAPI spec");

        // Ensure the docs directory exists
        let docs_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
        fs::create_dir_all(&docs_dir).expect("Failed to create docs directory");

        // Write the spec to the file
        let spec_path = docs_dir.join("openapi.yml");
        fs::write(&spec_path, spec).expect("Failed to write OpenAPI spec to file");

        println!("OpenAPI spec written to: {:?}", spec_path);
    }

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

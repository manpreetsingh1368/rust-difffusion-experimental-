use crate::config::Config;
use crate::errors::DiffusionError;
use crate::inference::pipeline::{GenerationParams, InferencePipeline};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    prompt: String,
    #[serde(default)]
    negative_prompt: Option<String>,
    #[serde(default = "default_steps")]
    num_inference_steps: i32,
    #[serde(default = "default_guidance")]
    guidance_scale: f64,
    #[serde(default = "default_size")]
    width: i32,
    #[serde(default = "default_size")]
    height: i32,
    seed: Option<i64>,
}

fn default_steps() -> i32 { 50 }
fn default_guidance() -> f64 { 7.5 }
fn default_size() -> i32 { 512 }

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    job_id: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images_base64: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<ResponseMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResponseMetadata {
    generation_time_seconds: f64,
    model_used: String,
    seed: i64,
    actual_steps: i32,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    model_loaded: bool,
    version: String,
    device: String,
}

struct AppState {
    pipeline: Arc<InferencePipeline>,
    config: Config,
}

async fn generate_image(
    req: web::Json<GenerateRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    info!("REST API: Generate request for prompt: {}", req.prompt);

    let job_id = uuid::Uuid::new_v4().to_string();

    let params = GenerationParams {
        prompt: req.prompt.clone(),
        negative_prompt: req.negative_prompt.clone(),
        num_inference_steps: req.num_inference_steps,
        guidance_scale: req.guidance_scale,
        width: req.width,
        height: req.height,
        seed: req.seed,
    };

    match data.pipeline.generate(params).await {
        Ok(result) => {
            // Convert to base64
            let images_base64: Vec<String> = result.images
                .iter()
                .map(|img| base64::encode(img))
                .collect();

            HttpResponse::Ok().json(GenerateResponse {
                job_id: job_id.clone(),
                status: "completed".to_string(),
                images_base64: Some(images_base64),
                metadata: Some(ResponseMetadata {
                    generation_time_seconds: result.generation_time,
                    model_used: "stable-diffusion-v1-5".to_string(),
                    seed: result.seed,
                    actual_steps: result.steps_taken,
                }),
                error: None,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(GenerateResponse {
                job_id,
                status: "error".to_string(),
                images_base64: None,
                metadata: None,
                error: Some(format!("Generation failed: {}", e)),
            })
        }
    }
}

async fn generate_image_binary(
    req: web::Json<GenerateRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    info!("REST API: Generate binary image for prompt: {}", req.prompt);

    let params = GenerationParams {
        prompt: req.prompt.clone(),
        negative_prompt: req.negative_prompt.clone(),
        num_inference_steps: req.num_inference_steps,
        guidance_scale: req.guidance_scale,
        width: req.width,
        height: req.height,
        seed: req.seed,
    };

    match data.pipeline.generate(params).await {
        Ok(result) => {
            if let Some(img_bytes) = result.images.first() {
                HttpResponse::Ok()
                    .content_type("image/png")
                    .body(img_bytes.clone())
            } else {
                HttpResponse::InternalServerError().body("No image generated")
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Generation failed: {}", e))
        }
    }
}

async fn health_check(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        model_loaded: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        device: data.config.model.device.clone(),
    })
}

pub async fn start_rest_server(
    config: Config,
    pipeline: InferencePipeline,
) -> Result<(), DiffusionError> {
    let addr = format!("{}:{}", config.server.rest_host, config.server.rest_port);
    
    info!("Starting REST API server on {}", addr);

    let app_state = web::Data::new(AppState {
        pipeline: Arc::new(pipeline),
        config: config.clone(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
            .route("/v1/generate", web::post().to(generate_image))
            .route("/v1/generate/binary", web::post().to(generate_image_binary))
    })
    .bind(&addr)
    .map_err(|e| DiffusionError::Internal(format!("Failed to bind server: {}", e)))?
    .run()
    .await
    .map_err(|e| DiffusionError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

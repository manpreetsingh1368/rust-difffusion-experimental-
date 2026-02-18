use anyhow::Result;

use tracing::{info, error};
use tracing_subscriber;
use std::sync::Arc;
use tokio::sync::Mutex;

mod config;
mod errors;
mod inference;
mod queue;
mod server;

use config::Config;
use inference::pipeline::{InferencePipeline, GenerationParams};
use tch::Device;

// Use the gRPC proto types directly to avoid type mismatch
use server::grpc::proto as grpc_proto;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("diffusion_server=debug,info")
        .init();

    info!("🚀 Starting Diffusion Server");

    // Load configuration
    let config = Config::from_file("config/default")
        .unwrap_or_else(|e| {
            info!("Could not load config file ({}), using defaults", e);
            Config::default()
        });

    info!("Configuration loaded");
    info!("  gRPC port: {}", config.server.grpc_port);
    info!("  REST port: {}", config.server.rest_port);
    info!("  Worker threads: {}", config.queue.worker_threads);

    // Initialize device
    let device = match config.model.device.as_str() {
        "cpu" => {
            info!("Using CPU device");
            Device::Cpu
        }
        d if d.starts_with("cuda") => {
            if tch::Cuda::is_available() {
                info!("✓ CUDA is available, using GPU");
                Device::Cuda(0)
            } else {
                info!("⚠ CUDA not available, falling back to CPU");
                Device::Cpu
            }
        }
        _ => {
            info!("Using CPU device (default)");
            Device::Cpu
        }
    };

    // Initialize inference pipeline
    let pipeline = InferencePipeline::new(config.inference.clone(), device)?;
    let pipeline = Arc::new(pipeline);

    // Initialize job queue with gRPC proto types
    let queue: queue::memory::MemoryQueue<
        grpc_proto::GenerateImageRequest,
        grpc_proto::GenerateImageResponse,
    > = queue::memory::MemoryQueue::new(config.queue.max_queue_size);
    let queue = Arc::new(queue);

    // Start worker threads
    info!("Starting {} worker threads", config.queue.worker_threads);
    for worker_id in 0..config.queue.worker_threads {
        let pipeline = Arc::clone(&pipeline);
        let queue = Arc::clone(&queue);

        tokio::spawn(async move {
            worker_loop(worker_id, pipeline, queue).await;
        });
    }

    // Start REST API server in background
    let rest_config = config.clone();
    let rest_pipeline = (*pipeline).clone();
    actix_web::rt::spawn(async move {
        if let Err(e) = server::start_rest_server(rest_config, rest_pipeline).await {
            error!("REST server error: {}", e);
        }
    });

    // Start gRPC server (blocking)
    info!("✓ Server initialization complete");
    server::start_grpc_server(
        config,
        (*pipeline).clone(),
        (*queue).clone(),
    ).await?;

    Ok(())
}

/// Worker loop that processes jobs from the queue
async fn worker_loop(
    worker_id: usize,
    pipeline: Arc<InferencePipeline>,
    queue: Arc<queue::memory::MemoryQueue<
        grpc_proto::GenerateImageRequest,
        grpc_proto::GenerateImageResponse,
    >>,
) {
    info!("Worker {} started", worker_id);

    loop {
        let job = queue.dequeue().await;

        if let Some(job) = job {
            info!("Worker {} processing job {}", worker_id, job.id);

            // Convert request to generation params
            let params = GenerationParams {
                prompt: job.request.prompt.clone(),
                negative_prompt: if job.request.negative_prompt.is_empty() {
                    None
                } else {
                    Some(job.request.negative_prompt.clone())
                },
                num_inference_steps: if job.request.num_inference_steps > 0 {
                    job.request.num_inference_steps
                } else {
                    50
                },
                guidance_scale: if job.request.guidance_scale > 0.0 {
                    job.request.guidance_scale
                } else {
                    7.5
                },
                width: if job.request.width > 0 {
                    job.request.width
                } else {
                    512
                },
                height: if job.request.height > 0 {
                    job.request.height
                } else {
                    512
                },
                seed: job.request.seed,
            };

            // Generate image
            let result = pipeline.generate(params).await;

            match result {
                Ok(generation_result) => {
                    info!(
                        "✓ Worker {} completed job {} in {:.2}s",
                        worker_id, job.id, generation_result.generation_time
                    );

                    let response = grpc_proto::GenerateImageResponse {
                        job_id: job.id.clone(),
                        images: generation_result.images,
                        status: "completed".to_string(),
                        metadata: Some(grpc_proto::GenerationMetadata {
                            generation_time_seconds: generation_result.generation_time,
                            model_used: "stable-diffusion-v1-5".to_string(),
                            seed: generation_result.seed,
                            actual_steps: generation_result.steps_taken,
                        }),
                    };

                    queue.update_status(&job.id, queue::memory::JobStatus::Completed).await;
                    let _ = job.response_tx.send(Ok(response));
                }
                Err(e) => {
                    error!("✗ Worker {} failed job {}: {}", worker_id, job.id, e);

                    queue.update_status(&job.id, queue::memory::JobStatus::Failed).await;
                    let _ = job.response_tx.send(Err(e));
                }
            }
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

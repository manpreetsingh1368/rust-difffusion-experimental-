use crate::config::Config;
use crate::errors::DiffusionError;
use crate::inference::pipeline::InferencePipeline;
use crate::queue::memory::MemoryQueue;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("diffusion");
}

use proto::diffusion_service_server::{DiffusionService, DiffusionServiceServer};
use proto::*;

type JobQueue = MemoryQueue<GenerateImageRequest, GenerateImageResponse>;

pub struct DiffusionGrpcService {
    config: Config,
    pipeline: InferencePipeline,
    queue: JobQueue,
}

impl DiffusionGrpcService {
    pub fn new(
        config: Config,
        pipeline: InferencePipeline,
        queue: JobQueue,
    ) -> Self {
        Self { config, pipeline, queue }
    }
}

#[tonic::async_trait]
impl DiffusionService for DiffusionGrpcService {
    async fn generate_image(
        &self,
        request: Request<GenerateImageRequest>,
    ) -> std::result::Result<Response<GenerateImageResponse>, Status> {
        let req = request.into_inner();
        
        info!("Received generation request: {}", req.prompt);
        
        // Enqueue job
        let (job_id, rx) = self.queue
            .enqueue(req)
            .await
            .map_err(|e| Status::resource_exhausted(format!("Queue full: {}", e)))?;
        
        // Wait for result
        let result = rx
            .await
            .map_err(|_| Status::internal("Worker dropped response"))?
            .map_err(|e| Status::internal(format!("Generation failed: {}", e)))?;
        
        Ok(Response::new(result))
    }
    
    async fn get_job_status(
        &self,
        request: Request<JobStatusRequest>,
    ) -> std::result::Result<Response<JobStatusResponse>, Status> {
        let req = request.into_inner();
        
        let status = self.queue.get_status(&req.job_id).await;
        
        match status {
            Some(s) => {
                let status_str = format!("{:?}", s);
                Ok(Response::new(JobStatusResponse {
                    job_id: req.job_id,
                    status: status_str,
                    result: None,
                    error: None,
                }))
            }
            None => Err(Status::not_found("Job not found")),
        }
    }
    
    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> std::result::Result<Response<HealthCheckResponse>, Status> {
        let queue_len = self.queue.queue_length().await;
        
        Ok(Response::new(HealthCheckResponse {
            status: "healthy".to_string(),
            model_loaded: true,
            queue_length: queue_len as i32,
            active_workers: self.config.queue.worker_threads as i32,
            system_info: Default::default(),
        }))
    }
}

pub async fn start_grpc_server(
    config: Config,
    pipeline: InferencePipeline,
    queue: JobQueue,
) -> Result<(), DiffusionError> {
    let addr = format!("{}:{}", config.server.grpc_host, config.server.grpc_port)
        .parse()
        .map_err(|e| DiffusionError::Config(format!("Invalid address: {}", e)))?;
    
    let service = DiffusionGrpcService::new(config, pipeline, queue);
    
    info!("Starting gRPC server on {}", addr);
    
    Server::builder()
        .add_service(DiffusionServiceServer::new(service))
        .serve(addr)
        .await
        .map_err(|e| DiffusionError::Internal(format!("Server error: {}", e)))?;
    
    Ok(())
}

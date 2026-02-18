use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub model: ModelConfig,
    pub inference: InferenceConfig,
    pub queue: QueueConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub grpc_host: String,
    pub grpc_port: u16,
    pub rest_host: String,
    pub rest_port: u16,
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: PathBuf,
    pub device: String,
    pub precision: String,
    pub cache_dir: PathBuf,
    pub warmup_on_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub default_steps: i32,
    pub default_guidance_scale: f64,
    pub default_width: i32,
    pub default_height: i32,
    pub max_width: i32,
    pub max_height: i32,
    pub max_steps: i32,
    pub safety_checker: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub backend: String,
    pub max_queue_size: usize,
    pub worker_threads: usize,
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("DIFFUSION").separator("__"))
            .build()?;
        
        Ok(settings.try_deserialize()?)
    }
    
    pub fn default() -> Self {
        Self {
            server: ServerConfig {
                grpc_host: "0.0.0.0".to_string(),
                grpc_port: 50051,
                rest_host: "0.0.0.0".to_string(),
                rest_port: 8080,
                max_concurrent_requests: 10,
                request_timeout_seconds: 300,
            },
            model: ModelConfig {
                model_path: PathBuf::from("./models/stable-diffusion-v1-5"),
                device: "cuda".to_string(),
                precision: "fp16".to_string(),
                cache_dir: PathBuf::from("./cache"),
                warmup_on_start: false,
            },
            inference: InferenceConfig {
                default_steps: 50,
                default_guidance_scale: 7.5,
                default_width: 512,
                default_height: 512,
                max_width: 1024,
                max_height: 1024,
                max_steps: 150,
                safety_checker: false,
            },
            queue: QueueConfig {
                backend: "memory".to_string(),
                max_queue_size: 1000,
                worker_threads: 2,
            },
        }
    }
}

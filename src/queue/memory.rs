use crate::errors::{DiffusionError, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

pub struct Job<Req, Res> {
    pub id: String,
    pub request: Req,
    pub response_tx: oneshot::Sender<Result<Res>>,
    pub status: JobStatus,
}

pub struct MemoryQueue<Req, Res> {
    queue: Arc<Mutex<VecDeque<Job<Req, Res>>>>,
    jobs: Arc<Mutex<HashMap<String, JobStatus>>>,
    max_size: usize,
}

impl<Req, Res> MemoryQueue<Req, Res> {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            max_size,
        }
    }
    
    pub async fn enqueue(
        &self,
        request: Req,
    ) -> Result<(String, oneshot::Receiver<Result<Res>>)> {
        let mut queue = self.queue.lock().await;
        
        if queue.len() >= self.max_size {
            return Err(DiffusionError::QueueFull);
        }
        
        let job_id = Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();
        
        let job = Job {
            id: job_id.clone(),
            request,
            response_tx: tx,
            status: JobStatus::Queued,
        };
        
        queue.push_back(job);
        
        let mut jobs = self.jobs.lock().await;
        jobs.insert(job_id.clone(), JobStatus::Queued);
        
        Ok((job_id, rx))
    }
    
    pub async fn dequeue(&self) -> Option<Job<Req, Res>> {
        let mut queue = self.queue.lock().await;
        let job = queue.pop_front()?;
        
        let mut jobs = self.jobs.lock().await;
        jobs.insert(job.id.clone(), JobStatus::Processing);
        
        Some(job)
    }
    
    pub async fn get_status(&self, job_id: &str) -> Option<JobStatus> {
        let jobs = self.jobs.lock().await;
        jobs.get(job_id).cloned()
    }
    
    pub async fn update_status(&self, job_id: &str, status: JobStatus) {
        let mut jobs = self.jobs.lock().await;
        jobs.insert(job_id.to_string(), status);
    }
    
    pub async fn queue_length(&self) -> usize {
        self.queue.lock().await.len()
    }
}

impl<Req, Res> Clone for MemoryQueue<Req, Res> {
    fn clone(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
            jobs: Arc::clone(&self.jobs),
            max_size: self.max_size,
        }
    }
}

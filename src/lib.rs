use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Critical,    // Run locally immediately
    NonCritical, // Queued during WAN outage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTask {
    pub task_id: String,
    pub patient_id: String,
    pub payload: String,
    pub priority: TaskPriority,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub status: String,
    pub processed_by: String,
}

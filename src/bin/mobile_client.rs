use mhealth_sovereignty::{HealthTask, TaskPriority, TaskResponse};
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let edge_url = "http://127.0.0.1:3030/submit";

    // 1. Send Critical Workload (Diagnostics / Emergency)
    let critical_task = HealthTask {
        task_id: "task-001".to_string(),
        patient_id: "patient-789".to_string(),
        payload: "ECG_ALERT_CRITICAL".to_string(),
        priority: TaskPriority::Critical,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    };

    println!("[CLIENT] Sending Critical Health Workload...");
    let res: TaskResponse = client.post(edge_url).json(&critical_task).send().await?.json().await?;
    println!("[CLIENT Response] {:?}", res);

    // 2. Send Non-Critical Workload (Background Sync / HD Image AI)
    let non_critical_task = HealthTask {
        task_id: "task-002".to_string(),
        patient_id: "patient-789".to_string(),
        payload: "HD_DIAGNOSTIC_IMAGE_SYNC".to_string(),
        priority: TaskPriority::NonCritical,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    };

    println!("[CLIENT] Sending Non-Critical Health Workload...");
    let res2: TaskResponse = client.post(edge_url).json(&non_critical_task).send().await?.json().await?;
    println!("[CLIENT Response] {:?}", res2);

    Ok(())
}

use serde::{Deserialize, Serialize};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Critical,    // Processed immediately on Edge Node
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

type TaskQueue = Arc<Mutex<Vec<HealthTask>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Dynamically bind port (defaults to 3030 if PORT env variable is not set)
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse()
        .expect("PORT must be a valid integer");

    let wan_online = Arc::new(AtomicBool::new(true));
    let queue: TaskQueue = Arc::new(Mutex::new(Vec::new()));

    // 1. Health check route for CRANE Cloud liveness probe
    let health_check = warp::path!("health").map(|| warp::reply::json(&"OK"));

    // 2. WAN status toggle route (Simulate WAN Partitioning)
    let wan_flag = wan_online.clone();
    let toggle_wan = warp::path!("wan" / "toggle").map(move || {
        let current = wan_flag.fetch_xor(true, Ordering::SeqCst);
        let status = if !current { "ONLINE" } else { "PARTITIONED" };
        println!("[ORCHESTRATOR] WAN state changed to: {}", status);
        warp::reply::json(&format!("WAN status is now {}", status))
    });

    // 3. Workload Ingestion Route
    let queue_filter = warp::any().map(move || queue.clone());
    let wan_filter = warp::any().map(move || wan_online.clone());

    let process_task = warp::path("submit")
        .and(warp::post())
        .and(warp::body::json())
        .and(queue_filter)
        .and(wan_filter)
        .and_then(handle_task);

    let routes = health_check.or(toggle_wan).or(process_task);

    println!("[EDGE ORCHESTRATOR] Server listening on 0.0.0.0:{}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

async fn handle_task(
    task: HealthTask,
    queue: TaskQueue,
    wan_online: Arc<AtomicBool>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let is_wan_up = wan_online.load(Ordering::SeqCst);

    if is_wan_up {
        println!("[NORMAL ROUTE] Task {} sent to Cloud/Edge", task.task_id);
        let resp = TaskResponse {
            task_id: task.task_id,
            status: "PROCESSED_NORMAL".to_string(),
            processed_by: "EDGE_OR_CLOUD".to_string(),
        };
        Ok(warp::reply::json(&resp))
    } else {
        match task.priority {
            TaskPriority::Critical => {
                println!("[WAN PARTITION] Task {} -> Executed Locally", task.task_id);
                let resp = TaskResponse {
                    task_id: task.task_id,
                    status: "EXECUTED_LOCALLY_CRITICAL".to_string(),
                    processed_by: "LOCAL_EDGE_ENGINE".to_string(),
                };
                Ok(warp::reply::json(&resp))
            }
            TaskPriority::NonCritical => {
                println!("[WAN PARTITION] Task {} -> Queued for Reconciliation", task.task_id);
                let mut q = queue.lock().await;
                q.push(task.clone());
                let resp = TaskResponse {
                    task_id: task.task_id,
                    status: "QUEUED_FOR_RECONCILIATION".to_string(),
                    processed_by: "LOCAL_EDGE_QUEUE".to_string(),
                };
                Ok(warp::reply::json(&resp))
            }
        }
    }
}

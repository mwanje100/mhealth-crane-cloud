use mhealth_sovereignty::{HealthTask, TaskPriority, TaskResponse};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

type TaskQueue = Arc<Mutex<Vec<HealthTask>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Read deployment port from environment variables (defaults to 3030 if not set)
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse()
        .expect("PORT must be a valid integer");

    let wan_online = Arc::new(AtomicBool::new(true));
    let queue: TaskQueue = Arc::new(Mutex::new(Vec::new()));

    // Health check endpoint for CRANE Cloud readiness/liveness probes
    let health_check = warp::path!("health").map(|| warp::reply::json(&"OK"));

    // Toggle WAN status endpoint (Simulates WAN Partitioning live)
    let wan_flag = wan_online.clone();
    let toggle_wan = warp::path!("wan" / "toggle").map(move || {
        let current = wan_flag.fetch_xor(true, Ordering::SeqCst);
        let status = if !current { "ONLINE" } else { "PARTITIONED" };
        println!("[ORCHESTRATOR] WAN state updated: {}", status);
        warp::reply::json(&format!("WAN status is now {}", status))
    });

    // Workload Ingestion Route
    let queue_filter = warp::any().map(move || queue.clone());
    let wan_filter = warp::any().map(move || wan_online.clone());

    let process_task = warp::path("submit")
        .and(warp::post())
        .and(warp::body::json())
        .and(queue_filter)
        .and(wan_filter)
        .and_then(handle_task);

    let routes = health_check.or(toggle_wan).or(process_task);

    println!("[CRANE CLOUD DEPLOYMENT] Server starting on 0.0.0.0:{}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

async fn handle_task(
    task: HealthTask,
    queue: TaskQueue,
    wan_online: Arc<AtomicBool>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let is_wan_up = wan_online.load(Ordering::SeqCst);

    if is_wan_up {
        println!("[NORMAL ROUTE] Task {} dispatched via Cloud/LAN", task.task_id);
        let resp = TaskResponse {
            task_id: task.task_id,
            status: "PROCESSED_NORMAL".to_string(),
            processed_by: "CLOUD_OR_EDGE_PRIMARY".to_string(),
        };
        Ok(warp::reply::json(&resp))
    } else {
        match task.priority {
            TaskPriority::Critical => {
                println!("[WAN PARTITION] Task {} -> Executing Locally", task.task_id);
                let resp = TaskResponse {
                    task_id: task.task_id,
                    status: "EXECUTED_LOCALLY_CRITICAL".to_string(),
                    processed_by: "LOCAL_EDGE_ENGINE".to_string(),
                };
                Ok(warp::reply::json(&resp))
            }
            TaskPriority::NonCritical => {
                println!("[WAN PARTITION] Task {} -> Enqueued for Reconciliation", task.task_id);
                let mut q = queue.lock().await;
                q.push(task.clone());
                let resp = TaskResponse {
                    task_id: task.task_id,
                    status: "QUEUED_FOR_RECONCILIATION".to_string(),
                    processed_by: "LOCAL_DELAY_QUEUE".to_string(),
                };
                Ok(warp::reply::json(&resp))
            }
        }
    }
}

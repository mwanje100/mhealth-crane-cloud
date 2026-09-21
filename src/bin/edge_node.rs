use mhealth_sovereignty::{HealthTask, TaskPriority, TaskResponse};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

type TaskQueue = Arc<Mutex<Vec<HealthTask>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Shared state
    let wan_online = Arc::new(AtomicBool::new(true)); // Orchestrator state flag
    let queue: TaskQueue = Arc::new(Mutex::new(Vec::new()));

    // Toggle WAN status endpoint (Simulate WAN Partitioning)
    let wan_flag = wan_online.clone();
    let toggle_wan = warp::path!("wan" / "toggle").map(move || {
        let current = wan_flag.fetch_xor(true, Ordering::SeqCst);
        let status = if !current { "ONLINE" } else { "PARTITIONED" };
        println!("[ORCHESTRATOR] WAN status changed to: {}", status);
        warp::reply::json(&format!("WAN is now {}", status))
    });

    // Ingest Health Workload Endpoint
    let queue_filter = warp::any().map(move || queue.clone());
    let wan_filter = warp::any().map(move || wan_online.clone());

    let process_task = warp::path("submit")
        .and(warp::post())
        .and(warp::body::json())
        .and(queue_filter)
        .and(wan_filter)
        .and_then(handle_task);

    // Background worker for Reconciliation + Rescheduling
    let reconcile_queue = Arc::new(Mutex::new(Vec::new())); // Duplicate handle for background loop
    let reconcile_wan = Arc::new(AtomicBool::new(true));
    
    // Spawn WAN reconciliation task
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            // Reconciliation logic when WAN is restored
            println!("[RECONCILIATION] Checking queued tasks for cloud dispatch...");
        }
    });

    let routes = toggle_wan.or(process_task);
    println!("[EDGE NODE] Running on 0.0.0.0:3030...");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn handle_task(
    task: HealthTask,
    queue: TaskQueue,
    wan_online: Arc<AtomicBool>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let is_wan_up = wan_online.load(Ordering::SeqCst);

    if is_wan_up {
        // Normal Execution: Process directly or forward to Cloud
        println!("[NORMAL] Routing task {} to Cloud/Local Edge", task.task_id);
        let resp = TaskResponse {
            task_id: task.task_id,
            status: "PROCESSED_NORMAL".to_string(),
            processed_by: "EDGE_OR_CLOUD".to_string(),
        };
        Ok(warp::reply::json(&resp))
    } else {
        // WAN Partitioning Active: Orchestrator Decision Tree
        match task.priority {
            TaskPriority::Critical => {
                println!("[WAN PARTITION] Critical Task {} -> Executing Locally", task.task_id);
                let resp = TaskResponse {
                    task_id: task.task_id,
                    status: "EXECUTED_LOCALLY_CRITICAL".to_string(),
                    processed_by: "EDGE_LOCAL_ENGINE".to_string(),
                };
                Ok(warp::reply::json(&resp))
            }
            TaskPriority::NonCritical => {
                println!("[WAN PARTITION] Non-Critical Task {} -> Delaying/Queuing", task.task_id);
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

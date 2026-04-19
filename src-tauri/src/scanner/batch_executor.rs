use std::sync::{Arc, Mutex};

use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::scanner::batch::BatchPlan;
use crate::scanner::engine::{CancelIntent, ScanEngine, ScanProgress};

pub struct BatchExecutor {
    engine: ScanEngine,
}

impl BatchExecutor {
    pub fn new(engine: ScanEngine) -> Self {
        Self { engine }
    }

    pub async fn execute_local_batch(
        &self,
        plan: &BatchPlan,
        conn: Arc<Mutex<rusqlite::Connection>>,
        cancel_token: CancellationToken,
        cancel_intent: Arc<Mutex<CancelIntent>>,
        app: &AppHandle,
    ) -> Result<ScanProgress, String> {
        self.engine
            .run_scan_range(
                &plan.task_id,
                &plan.run_id,
                plan.start_index,
                plan.end_index,
                conn,
                cancel_token,
                cancel_intent,
                app,
            )
            .await
    }
}

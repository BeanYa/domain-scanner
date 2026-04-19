use std::sync::{Arc, Mutex};

use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::scanner::batch::BatchPlan;
use crate::scanner::batch_executor::BatchExecutor;
use crate::scanner::engine::{CancelIntent, ScanProgress};

pub struct LocalEmbeddedWorker {
    executor: BatchExecutor,
}

impl LocalEmbeddedWorker {
    pub fn new(executor: BatchExecutor) -> Self {
        Self { executor }
    }

    pub async fn execute_batch(
        &self,
        plan: &BatchPlan,
        conn: Arc<Mutex<rusqlite::Connection>>,
        cancel_token: CancellationToken,
        cancel_intent: Arc<Mutex<CancelIntent>>,
        app: &AppHandle,
    ) -> Result<ScanProgress, String> {
        self.executor
            .execute_local_batch(plan, conn, cancel_token, cancel_intent, app)
            .await
    }
}

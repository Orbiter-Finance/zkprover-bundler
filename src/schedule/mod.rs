use crate::service::pool::batch_received_txs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::error;

lazy_static::lazy_static! {
    static ref DO_BATCH_RECEIVED_TXS_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

pub async fn do_batch_received_txs() {
    let _lock_guard = DO_BATCH_RECEIVED_TXS_LOCK.lock().await;

    let result = batch_received_txs().await;
    match result {
        Err(err) => error!("Job batch_received_txs failed: {}", err),
        _ => {}
    }
}

pub async fn start_schedules() {
    let sched = JobScheduler::new().await.unwrap();

    // Job batch_received_txs
    sched
        .add(Job::new_async("1/5 * * * * *", |_, _| Box::pin(do_batch_received_txs())).unwrap())
        .await
        .unwrap();

    sched.start().await.unwrap();
}

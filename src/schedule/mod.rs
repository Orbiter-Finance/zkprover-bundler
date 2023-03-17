use crate::service::pool::batch_received_txs;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn start_schedules() {
    let sched = JobScheduler::new().await.unwrap();

    // Job batch_received_txs
    sched
        .add(
            Job::new_async("1/5 * * * * *", |_, _| {
                Box::pin(async {
                    let result = batch_received_txs().await;
                    match result {
                        Err(err) => println!("Job batch_received_txs failed: {}", err.to_string()),
                        _ => {}
                    }
                })
            })
            .unwrap(),
        )
        .await
        .unwrap();

    sched.start().await.unwrap();
}
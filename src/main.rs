use std::time::Duration;

use tokio::{join, select, spawn, sync::oneshot, time::sleep};
use tracing::{info, subscriber::set_global_default};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .finish();
    set_global_default(subscriber)?;

    info!("Hello, world!");

    let (mut sender, receiver) = oneshot::channel();
    let (decode_sender, decode_receiver) = oneshot::channel();

    spawn(async {
        select! {
            val = async_io() => {
                info!("sending async_io");
                let _ = sender.send(val);
            }
            _ = sender.closed() => {
                info!("Sender closed, exiting...");
            }
        }
    });

    spawn(async {
        sleep(Duration::from_secs(20)).await;
        info!("Sending goodbye...");
        let _ = decode_sender.send("Goodbye, world!");
    });

    let parell = join!(request_date(3), request_date(5));

    info!("Parell: {:?}", parell);

    info!("Waiting for signals...");

    select! {
        _ = async {
            let mut ticks = 0;
            loop {
                ticks += 1;
                sleep(Duration::from_secs(1)).await;
                info!("Still waiting... {}", ticks);
            }
        } => {}
        _ = tokio::signal::ctrl_c() => {
            info!("\nCtrl-C received, exiting...");
        }
        val = receiver => {
            info!("Received: {}", val.unwrap());
        }
        val = decode_receiver => {
            info!("Received codeder msg: {}", val.unwrap());
        }
    }

    condition_if().await;

    info!("Exiting...");

    Ok(())
}

async fn request_date(delay: u64) -> String {
    info!("Requesting date...{}", delay);
    sleep(Duration::from_secs(delay)).await;
    info!("Date received! {}", delay);
    format!("《{}》request_date", delay)
}

async fn async_io() -> String {
    sleep(Duration::from_secs(8)).await;
    "Hello, world!".to_string()
}

async fn condition_if() {
    let mut attempt_count = 0;

    loop {
        attempt_count += 1;
        let time_condition = attempt_count <= 3;

        select! {
            _ = sleep(Duration::from_secs(1)), if time_condition => {
                info!("[Attempt {}] 超时发射！", attempt_count);
            }
            _ = async {
                info!("[Attempt {}] 执行主任务！", attempt_count);
                sleep(Duration::from_secs(2)).await;
                info!("[Attempt {}] 主任务完成！", attempt_count);
            } => {
                info!("condition met!");
                break;
            }
        }
        info!("loop = {}", attempt_count);
    }
}

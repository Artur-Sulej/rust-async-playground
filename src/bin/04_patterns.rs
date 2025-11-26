//! 04_patterns.rs
//!
//! This example demonstrates advanced control flow patterns in async Rust.
//!
//! Concepts:
//! 1. `tokio::select!`: Waiting on multiple branches, handling the first one that completes.
//! 2. `tokio::time::timeout`: Putting a time limit on a future.
//! 3. Cancellation: Dropping a future cancels it.
//!
//! Scenarios:
//! - Racing two tasks against each other.
//! - Waiting for a task with a timeout.
//! - Handling user input (simulated) vs background work.

use tokio::sync::oneshot;
use tokio::time::{Duration, sleep, timeout};

#[tokio::main]
async fn main() {
    // 1. Racing tasks with `tokio::select!`
    // Whichever branch finishes first runs its block. The other is cancelled (dropped).
    println!("--- Race Example ---");
    let t1 = async {
        sleep(Duration::from_millis(100)).await;
        "Task 1 won"
    };
    let t2 = async {
        sleep(Duration::from_millis(200)).await;
        "Task 2 won"
    };

    tokio::select! {
        val = t1 => println!("Race result: {}", val),
        val = t2 => println!("Race result: {}", val),
    }

    // 2. Timeout
    // `timeout` wraps a future. If it takes too long, it returns Err(Elapsed).
    println!("\n--- Timeout Example ---");
    let slow_task = async {
        println!("Slow task started...");
        sleep(Duration::from_secs(2)).await;
        "Finished!"
    };

    match timeout(Duration::from_secs(1), slow_task).await {
        Ok(val) => println!("Task completed: {}", val),
        Err(_) => println!("Task timed out!"),
    }

    // 3. Cancellation via `oneshot` channel
    // We can use a oneshot channel to send a signal to stop a task manually.
    println!("\n--- Cancellation Example ---");
    let (tx, mut rx) = oneshot::channel();

    let pinger = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = sleep(Duration::from_millis(500)) => {
                    println!("Ping...");
                }
                _ = &mut rx => {
                    println!("Stop signal received!");
                    break;
                }
            }
        }
    });

    sleep(Duration::from_secs(2)).await;
    println!("Sending stop signal...");
    let _ = tx.send(()); // Send unit to signal stop
    pinger.await.unwrap();

    println!("Main finished.");
}

//! 05_mini_app.rs
//!
//! This example puts everything together into a mini "Job Processing" application.
//!
//! Architecture:
//! - `Job`: A struct representing work to be done.
//! - `JobGenerator`: A task that creates jobs and pushes them to a queue.
//! - `JobQueue`: A `mpsc` channel.
//! - `Dispatcher`: The main loop that pulls jobs and spawns workers.
//! - `Concurrency Control`: Using `tokio::sync::Semaphore` to limit the number of active workers.
//! - `Graceful Shutdown`: Handling Ctrl+C to stop the generator and wait for workers.
//!
//! Concepts:
//! - **Backpressure**: If the workers are slow, the semaphore fills up. The dispatcher waits.
//!   Then the channel fills up. Then the generator waits.
//! - **Semaphore**: Limiting concurrency (e.g., max 3 concurrent jobs).
//! - **Graceful Shutdown**: Using `CancellationToken` (or just boolean flags/channels) to stop.
//!   Here we use a simple `broadcast` channel for cancellation.

use rand::Rng;
use std::sync::Arc;
use tokio::sync::{Semaphore, broadcast, mpsc};
use tokio::time::{Duration, sleep};

#[derive(Debug, Clone)]
struct Job {
    id: u32,
    difficulty: u64, // Simulated time in ms
}

#[tokio::main]
async fn main() {
    println!("--- Job Processor App Starting ---");

    // 1. Setup Channels
    // Channel capacity = 10. If we have >10 pending jobs, generator will sleep.
    let (tx, mut rx) = mpsc::channel::<Job>(10);

    // Broadcast channel for shutdown signal to workers (if we needed to cancel them mid-job).
    // In this pattern, we might just let them finish.
    // But let's keep it to show how to broadcast "stop".
    let (shutdown_tx, _) = broadcast::channel(1);

    // 2. Concurrency Control
    // We want at most 3 concurrent jobs.
    let semaphore = Arc::new(Semaphore::new(3));

    // 3. Spawn Job Generator
    let tx_clone = tx.clone();
    let generator_handle = tokio::spawn(async move {
        let mut id = 0;
        loop {
            let difficulty = rand::thread_rng().gen_range(500..1500);
            let job = Job { id, difficulty };

            println!("[Generator] Sending Job {}...", id);
            if tx_clone.send(job).await.is_err() {
                println!("[Generator] Receiver dropped, stopping.");
                break;
            }
            id += 1;
            // Generate faster than workers to demonstrate backpressure
            sleep(Duration::from_millis(100)).await;
        }
    });

    // 4. Dispatcher Loop (Main Task)
    // We run the dispatcher in a separate task or just in main.
    // Let's run it in main for simplicity, wrapped in a select with Ctrl+C.

    println!("Press Ctrl+C to stop...");

    // We need to track spawned tasks to wait for them at the end.
    // A `JoinSet` is perfect for this (Tokio 1.20+), but let's use a Vec for simplicity/older versions compatibility
    // or just rely on the semaphore to know when we are drained?
    // Actually, `JoinSet` is great. Let's use it if we assume recent Tokio.
    // But to keep it "classic", let's just use a simple counter or vector of handles.
    // Since we detach them (fire and forget), we can't easily `await` them all unless we store handles.
    // Let's store handles.
    let mut worker_handles = vec![];

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n[Dispatcher] Ctrl+C received! Shutting down...");
                break;
            }
            // Wait for a job AND a permit
            // We need to be careful. If we get a job but no permit, we shouldn't drop the job.
            // Correct pattern: Acquire permit first?
            // If we acquire permit, then wait for job... if no job comes (shutdown), we drop permit.
            //
            // But `rx.recv()` is cancel-safe. `semaphore.acquire()` is cancel-safe.
            //
            // Let's do:
            // 1. Acquire permit.
            // 2. Recv job.
            //
            // Wait, if we wait for permit, we block Ctrl+C?
            // No, we should put permit acquisition inside `select!`?
            // `semaphore.acquire()` is an async future.

            permit_result = semaphore.clone().acquire_owned() => {
                let permit = match permit_result {
                    Ok(p) => p,
                    Err(_) => break, // Semaphore closed
                };

                // Now we have a permit. Try to get a job.
                // We use `rx.recv()` here.
                // NOTE: If generator is slow, we hold the permit while waiting for job.
                // This is fine, it just means "we are ready to work".
                //
                // BUT, if we want to handle Ctrl+C while waiting for a job?
                // We need another select! inside? Or just break the outer loop structure?

                // Let's try to get a job.
                match rx.recv().await {
                    Some(job) => {
                        let mut shutdown_rx = shutdown_tx.subscribe();
                        let handle = tokio::spawn(async move {
                            println!("[Worker] Got Job {} (Difficulty: {}ms)", job.id, job.difficulty);

                            // Simulate work with cancellation support
                            tokio::select! {
                                _ = sleep(Duration::from_millis(job.difficulty)) => {
                                    println!("[Worker] Job {} Finished.", job.id);
                                }
                                _ = shutdown_rx.recv() => {
                                    println!("[Worker] Job {} Cancelled!", job.id);
                                }
                            }

                            // Permit is dropped here, allowing a new worker to start.
                            drop(permit);
                        });
                        worker_handles.push(handle);
                    }
                    None => {
                        println!("[Dispatcher] Queue is empty and closed.");
                        break;
                    }
                }
            }
        }
    }

    // 5. Shutdown Sequence
    println!("[Shutdown] Stopping generator...");
    generator_handle.abort();

    println!("[Shutdown] notifying workers...");
    let _ = shutdown_tx.send(());

    println!("[Shutdown] Waiting for active workers to finish/cancel...");
    for handle in worker_handles {
        let _ = handle.await;
    }

    println!("[Shutdown] All done.");
}

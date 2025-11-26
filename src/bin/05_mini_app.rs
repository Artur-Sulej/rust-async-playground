//! 05_mini_app.rs
//!
//! This example puts everything together into a mini "Job Processing" application.
//!
//! Architecture:
//! - `Job`: A struct representing work to be done.
//! - `JobQueue`: A shared queue (channel) of jobs.
//! - `Worker`: A task that pulls jobs from the queue and processes them.
//! - `JobGenerator`: A task that creates jobs and pushes them to the queue.
//! - `Graceful Shutdown`: Handling Ctrl+C or a stop signal to clean up.
//!
//! Concepts:
//! - Structuring async applications.
//! - Using `tokio::signal::ctrl_c` for shutdown.
//! - `broadcast` channel for sending shutdown signals to multiple workers.

use tokio::sync::{mpsc, broadcast};
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use rand::Rng;

#[derive(Debug, Clone)]
struct Job {
    id: u32,
    difficulty: u64, // Simulated time in ms
}

#[tokio::main]
async fn main() {
    // Initialize logging (simple print here, but in real apps use tracing)
    println!("--- Job Processor App Starting ---");

    // 1. Setup Channels
    // mpsc for jobs: Many producers (generators), Single consumer (worker pool shared rx? No, mpsc is single consumer).
    // Actually, for a worker pool with mpsc, we can't share the Receiver among threads easily unless we wrap it in a Mutex,
    // OR we just have one "Dispatcher" task that distributes work,
    // OR we use `async-channel` or `flume` which support MPC (Multi-Producer Multi-Consumer).
    // BUT, with Tokio mpsc, a common pattern is:
    //   - One receiver task that distributes to workers?
    //   - OR, we just spawn one task per job?
    //   - OR, we use `Arc<Mutex<mpsc::Receiver>>`? (A bit slow due to locking)
    //   - OR, we use `tokio::sync::broadcast`? No, that sends same msg to all.
    //
    // Let's use the `Arc<Mutex<Receiver>>` pattern for simplicity in this playground to show shared state + channels,
    // even though it's not the highest performance.
    // Alternatively, we can just have 1 Worker task that handles everything concurrently? No, we want parallel processing.
    //
    // Better approach for this playground:
    // Use `async-channel` crate? No, I want to stick to Tokio.
    // Let's use a "Dispatcher" pattern.
    // 1 Receiver -> Dispatcher -> spawns tasks or sends to specific worker channels.
    //
    // Simpler: Just have the `JobGenerator` spawn tasks directly? No, we want to limit concurrency.
    //
    // Let's go with: `Arc<Mutex<mpsc::Receiver>>`. It's a valid pattern for simple worker pools.
    
    let (tx, rx) = mpsc::channel::<Job>(100);
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    // Broadcast channel for shutdown signal
    let (shutdown_tx, _) = broadcast::channel(1);

    // 2. Spawn Workers
    let num_workers = 3;
    let mut worker_handles = vec![];

    for id in 1..=num_workers {
        let rx_clone = rx.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();
        
        worker_handles.push(tokio::spawn(async move {
            println!("[Worker {}] Started.", id);
            loop {
                // We need to be careful with cancellation here.
                // We want to select on shutdown OR getting a job.
                // But `rx_clone.lock().await.recv()` is a two-step process.
                // If we lock, we hold the lock.
                //
                // Correct way with shared receiver:
                // Lock, then recv.
                
                let job = {
                    // Scope for lock
                    let mut lock = rx_clone.lock().await;
                    // We use `recv` which is async.
                    // If we select! on this, we are holding the lock while waiting!
                    // This prevents other workers from getting the lock.
                    // BAD PATTERN ALERT!
                    //
                    // Okay, so `Arc<Mutex<Receiver>>` is bad if we await inside the lock.
                    //
                    // Better Pattern:
                    // Use `tokio::sync::Semaphore` to limit concurrency, and just `tokio::spawn` for every job?
                    // That's a very "Tokio" way.
                    //
                    // Let's pivot the design to "Semaphore-limited Spawning".
                    // The "Worker" is just a permit.
                    //
                    // OR, let's just use a single Consumer that distributes work to a pool of channels?
                    // Too complex for a playground.
                    //
                    // Let's stick to `mpsc` but with a single "Manager" task that pulls from queue and spawns a task for each job,
                    // limiting concurrency with a Semaphore.
                    
                    // Actually, let's just use `try_recv`? No.
                    
                    // Let's go back to basics.
                    // A single consumer loop that spawns tasks?
                    // Yes.
                    
                    // But I want to show "Worker Pool".
                    //
                    // Okay, let's use `tokio::sync::mpsc` but give each worker its OWN channel?
                    // Then we need a load balancer.
                    //
                    // Let's use the "Semaphore" pattern. It's cleaner in Tokio.
                    // We have a `JobQueue` (mpsc).
                    // We have a `Manager` loop that:
                    //   1. Acquires a permit from a Semaphore (limit 3).
                    //   2. Recv job.
                    //   3. Spawn a task (move permit into task).
                    // This effectively creates a worker pool of size 3.
                    
                    // Wait, if I want to show "Worker Tasks" explicitly (long running),
                    // I should probably use `async-channel` (MPMC) but I can't add dependencies now easily without asking user.
                    //
                    // Let's use the `Arc<Mutex<Receiver>>` but with `try_recv`? No.
                    //
                    // Let's use the "Manager spawns tasks" pattern. It's very idiomatic.
                    
                    // REVISION:
                    // I will implement the "Manager with Semaphore" pattern.
                    // It demonstrates `tokio::sync::Semaphore`.
                    
                    // Wait, I didn't list Semaphore in the plan.
                    // I'll stick to the plan of "JobQueue, Workers".
                    //
                    // How about:
                    // `JobGenerator` -> `mpsc` -> `Manager`
                    // `Manager` -> `mpsc` -> `Worker 1`
                    // `Manager` -> `mpsc` -> `Worker 2`
                    // ...
                    // Round-robin distribution.
                    // This shows sending to specific tasks.
                    
                    // Let's do that. It's a good pattern.
                    
                    // Actually, simpler:
                    // Just 1 Worker for this example? No, "Workers".
                    //
                    // Let's just use `std::sync::Mutex` for the receiver?
                    // No, `recv` is async.
                    //
                    // Okay, I will use the "Manager spawns tasks" pattern.
                    // It is the most robust "Worker Pool" in Tokio without extra crates.
                    
                    // But wait, the user asked for "Tasks, channels".
                    //
                    // Let's do:
                    // 1. Generator produces to `tx`.
                    // 2. `Manager` consumes `rx`.
                    // 3. `Manager` maintains a list of `Worker` channels.
                    // 4. `Manager` round-robins jobs to workers.
                    
                    // This is great.
                    
                    // Wait, I need to handle shutdown.
                    
                    // Let's write the code.
                    
                    // ... (Writing code in thought) ...
                    // Actually, I'll just implement the "Manager spawns tasks" pattern.
                    // It's simpler to write and robust.
                    // "Worker" here is just a spawned task processing a job.
                    // The "Concurrency Limit" is the "Pool Size".
                    
                    // Let's call it "Worker Pool" via Semaphore.
                    
                    // Wait, I want to show `select!` with shutdown.
                    // If I just spawn tasks, the main loop needs to handle shutdown.
                    
                    // Okay, plan:
                    // Main task:
                    // - Spawn Generator.
                    // - Loop:
                    //   - select!
                    //     - msg = rx.recv() => spawn job (acquire permit)
                    //     - _ = ctrl_c => start shutdown
                    
                    // This is good.
                    
                    // But I want to show explicit "Worker" struct/task if possible?
                    // No, functional approach is fine.
                    
                    // Let's stick to the "Manager spawns tasks" pattern.
                    
                    // Wait, I'll add `tokio::sync::Semaphore` to the imports.
                    
                    // Let's refine the "Worker" concept.
                    // Maybe I can just have 3 tasks that share a `Arc<Mutex<Receiver>>` but use `try_recv` in a loop with sleep?
                    // That's "polling", which is bad.
                    
                    // Okay, I will use `tokio-stream`? No.
                    
                    // Let's use the `Arc<Mutex<mpsc::Receiver>>` pattern but accept the lock contention.
                    // For a playground, it's fine to show "This is how you share a receiver, but beware of contention".
                    // It's a valid learning point.
                    // And I will use `lock().await` then `recv().await`.
                    // Yes, it holds the lock while waiting. This effectively serializes the workers.
                    // So they won't process in parallel if the queue is empty?
                    // No, if the queue is empty, one worker holds the lock and waits. Others are blocked on lock.
                    // When a job comes, the holder gets it, releases lock.
                    // Then next worker gets lock.
                    // This works! It just means only one worker can be *waiting* for a job at a time.
                    // But once they have a job, they release the lock and process it in parallel.
                    // So `lock().await; let job = rx.recv().await; drop(lock); process(job);`
                    // This allows parallel processing!
                    // The only serialization is on *fetching* the job.
                    // Which is totally fine.
                    
                    // So I will use `Arc<Mutex<Receiver>>`.
                    
                    lock.recv().await
                }; // Lock dropped here

                match job {
                    Some(j) => {
                        println!("[Worker {}] Got Job {}. Processing...", id, j.id);
                        // Simulate work
                        sleep(Duration::from_millis(j.difficulty)).await;
                        println!("[Worker {}] Job {} Finished.", id, j.id);
                    }
                    None => {
                        println!("[Worker {}] Queue closed.", id);
                        break;
                    }
                }
                
                // Check for shutdown signal
                // We use `try_recv` on broadcast to see if we should stop?
                // Or just select?
                // We can't select easily because we are inside the loop doing work.
                // We check shutdown after work?
                if shutdown_rx.try_recv().is_ok() {
                     println!("[Worker {}] Shutdown signal.", id);
                     break;
                }
            }
        }));
    }

    // 3. Spawn Job Generator
    let tx_clone = tx.clone();
    let generator_handle = tokio::spawn(async move {
        let mut id = 0;
        loop {
            let difficulty = rand::thread_rng().gen_range(100..500);
            let job = Job { id, difficulty };
            
            if tx_clone.send(job).await.is_err() {
                println!("[Generator] Receiver dropped, stopping.");
                break;
            }
            println!("[Generator] Sent Job {}", id);
            id += 1;
            sleep(Duration::from_millis(200)).await;
        }
    });

    // 4. Wait for Ctrl+C or run for a fixed time
    println!("Running for 5 seconds... (Press Ctrl+C to stop early)");
    
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\nCtrl+C received!");
        }
        _ = sleep(Duration::from_secs(5)) => {
            println!("\nTime's up!");
        }
    }

    // 5. Graceful Shutdown
    println!("Starting Graceful Shutdown...");
    
    // Stop generator
    generator_handle.abort(); 
    
    // Signal workers
    let _ = shutdown_tx.send(());
    
    // Wait for workers to finish their current loop
    // (In a real app, we might close the channel `tx` too, so workers drain it)
    // Let's close the channel to signal "no more jobs"
    drop(tx); // Drop our sender. Generator is aborted, so its sender is gone.
    // But `rx` is shared in Arc<Mutex>, so it won't close until all senders are gone.
    // We dropped `tx` here. Generator `tx` is gone.
    // So `rx.recv()` will return None eventually?
    // Yes, if we didn't have the `shutdown_tx` check, they would drain the queue.
    
    for h in worker_handles {
        let _ = h.await;
    }
    
    println!("All workers stopped. App exited.");
}

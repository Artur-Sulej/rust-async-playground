//! 02_channels.rs
//!
//! This example demonstrates message passing between tasks using Tokio channels.
//!
//! Concepts:
//! 1. `tokio::sync::mpsc`: Multi-Producer, Single-Consumer channel.
//! 2. `tx.send()`: Sending messages (async).
//! 3. `rx.recv()`: Receiving messages (async).
//! 4. Moving `Sender`s into tasks.
//!
//! Pattern:
//! This implements a simple "Actor" or "Worker" pattern where one task (the manager)
//! spawns a worker task and sends it commands.

use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum Command {
    Print(String),
    Add(i32, i32),
    Shutdown,
}

#[tokio::main]
async fn main() {
    // Create a channel with a buffer size of 32.
    // If the buffer fills up, sending will wait until there's space.
    let (tx, mut rx) = mpsc::channel(32);

    // Spawn the "Worker" task.
    // We move `rx` (the receiver) into this task.
    let worker_handle = tokio::spawn(async move {
        println!("[Worker] Started. Waiting for commands...");

        // Loop and receive messages until the channel is closed or we break.
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Print(msg) => {
                    println!("[Worker] Printing: {}", msg);
                }
                Command::Add(a, b) => {
                    println!("[Worker] Adding {} + {} = {}", a, b, a + b);
                    // Simulate some processing time
                    sleep(Duration::from_millis(100)).await;
                }
                Command::Shutdown => {
                    println!("[Worker] Shutdown command received. Bye!");
                    break;
                }
            }
        }
        println!("[Worker] Task finished.");
    });

    // The "Manager" (main task) sends commands.
    // We can clone `tx` if we had multiple producers, but here we just use the original.
    
    println!("[Manager] Sending Print command...");
    tx.send(Command::Print("Hello from main!".to_string())).await.unwrap();

    println!("[Manager] Sending Add commands...");
    tx.send(Command::Add(10, 20)).await.unwrap();
    tx.send(Command::Add(5, 5)).await.unwrap();

    // Simulate doing other things
    sleep(Duration::from_millis(500)).await;

    println!("[Manager] Sending Shutdown command...");
    tx.send(Command::Shutdown).await.unwrap();

    // Wait for the worker to finish
    worker_handle.await.unwrap();
    
    println!("[Manager] All done.");
}

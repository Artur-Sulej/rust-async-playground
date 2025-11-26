# Async Rust Playground

A realistic, educational playground for learning async Rust with Tokio.
This project demonstrates common patterns, best practices, and real-world scenarios.

## Getting Started

1. **Clone the repo** (if you haven't already).
2. **Run the examples** in order.

## Examples

### 1. Basics (`01_basics.rs`)
Learn the syntax of `async`/`await`, how to spawn tasks, and how to wait for them.
```bash
cargo run --bin 01_basics
```
**Key Concepts:** `tokio::spawn`, `tokio::join!`, `Send` trait.

### 2. Channels (`02_channels.rs`)
Message passing between tasks using `mpsc` (Multi-Producer, Single-Consumer) channels.
```bash
cargo run --bin 02_channels
```
**Key Concepts:** `tokio::sync::mpsc`, Actor/Worker pattern.

### 3. Shared State (`03_shared_state.rs`)
How to safely share mutable state across tasks using `Arc` and `Mutex`/`RwLock`.
```bash
cargo run --bin 03_shared_state
```
**Key Concepts:** `std::sync::Arc`, `tokio::sync::Mutex`, `tokio::sync::RwLock`.

### 4. Patterns (`04_patterns.rs`)
Advanced control flow: racing tasks, timeouts, and cancellation.
```bash
cargo run --bin 04_patterns
```
**Key Concepts:** `tokio::select!`, `tokio::time::timeout`, `oneshot` channels.

### 5. Mini App (`05_mini_app.rs`)
Putting it all together: A Job Processing system with a generator, a worker pool, and graceful shutdown.
```bash
cargo run --bin 05_mini_app
```
**Key Concepts:** Application structure, `tokio::signal::ctrl_c`, `broadcast` channels, graceful shutdown.

## Dependencies

- **tokio**: The async runtime.
- **anyhow**: Easy error handling.
- **tracing**: Structured logging (used simply here).
- **rand**: For simulation randomness.

## Further Reading

- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/): The definitive guide to the "why" and "how" of async (Futures, Pin, Waker).
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial): Excellent official tutorial for the runtime.

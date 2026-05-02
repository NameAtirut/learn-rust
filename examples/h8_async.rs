//! Hour 8 — async / await with Tokio + capstone.
//!
//! `async fn` returns a `Future` — a state machine you must `.await` to drive.
//! A *runtime* (Tokio is the de facto choice) polls those futures, multiplexing
//! many concurrent tasks onto a small pool of OS threads. Async is the right
//! tool for I/O-bound concurrency (network, files, timers); for CPU-bound work
//! use threads + rayon (Hour 7).
//!
//! Mental model:
//!   - `async fn foo()` is a function that returns `impl Future<Output = T>`.
//!   - `.await` says "yield until this future is ready". The runtime parks
//!     the task and resumes it later.
//!   - `tokio::spawn` schedules a future as an independent task — like
//!     `thread::spawn`, but cheap (microseconds, not milliseconds).
//!
//! Run with: cargo run --example h8_async
//!
//! Capstone: a concurrent URL fetcher with bounded parallelism. It exercises
//! ownership, Result/?, Arc, async, semaphores, and joining task results.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::sync::Semaphore;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    intro().await;
    spawn_and_join().await;
    sleep_concurrently().await?;
    capstone_fetch_titles().await?;
    Ok(())
}

// --- the simplest possible async ---
async fn intro() {
    async fn say(msg: &str) {
        println!("[intro] {msg}");
    }
    say("hello from async").await;       // .await is required to actually run it
}

// --- tokio::spawn for independent tasks ---
async fn spawn_and_join() {
    let h = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        7u32
    });
    let v = h.await.expect("task panicked");
    println!("[spawn] task returned {v}");
}

// --- doing things in parallel with `join!` ---
async fn sleep_concurrently() -> Result<()> {
    let t = Instant::now();
    // tokio::join! runs the futures concurrently on the SAME task.
    // (For independent tasks across the runtime, use tokio::spawn.)
    let (a, b, c) = tokio::join!(
        tokio::time::sleep(Duration::from_millis(50)),
        tokio::time::sleep(Duration::from_millis(50)),
        tokio::time::sleep(Duration::from_millis(50)),
    );
    let _ = (a, b, c);
    println!("[join] 3x 50ms sleeps in {}ms", t.elapsed().as_millis());
    // Should print ~50, not ~150 — they ran concurrently.
    Ok(())
}

// --- capstone: concurrent fetch with bounded parallelism ---
//
// Goal: GET a list of URLs, extract the <title>, print them. Cap parallelism
// at MAX_INFLIGHT requests using a Semaphore. Errors don't kill the whole
// program — each URL gets its own Result.

const MAX_INFLIGHT: usize = 4;

async fn fetch_title(client: &reqwest::Client, url: &str) -> Result<String> {
    let body = client
        .get(url)
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .with_context(|| format!("GET {url}"))?
        .text()
        .await
        .with_context(|| format!("read body of {url}"))?;
    let title = extract_title(&body).unwrap_or_else(|| "<no title>".into());
    Ok(title)
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let after_open = start + html[start..].find('>')? + 1;
    let end_rel = lower[after_open..].find("</title>")?;
    Some(html[after_open..after_open + end_rel].trim().to_string())
}

async fn capstone_fetch_titles() -> Result<()> {
    let urls = vec![
        "https://www.rust-lang.org/",
        "https://doc.rust-lang.org/book/",
        "https://crates.io/",
        "https://tokio.rs/",
        "https://example.com/",
    ];

    let client = reqwest::Client::builder()
        .user_agent("learn-rust/0.1")
        .build()?;
    let sem = Arc::new(Semaphore::new(MAX_INFLIGHT));

    let mut handles = Vec::with_capacity(urls.len());
    for url in urls {
        let permit = Arc::clone(&sem).acquire_owned().await?;  // backpressure
        let client = client.clone();                           // cheap, internal Arc
        handles.push(tokio::spawn(async move {
            let result = fetch_title(&client, url).await;
            drop(permit);                                      // release slot
            (url, result)
        }));
    }

    for h in handles {
        let (url, result) = h.await.expect("task panicked");
        match result {
            Ok(title) => println!("[capstone] {url} -> {title}"),
            Err(e)    => println!("[capstone] {url} FAILED: {e:#}"),
        }
    }
    Ok(())
}

// =================== EXERCISES ===================
//
// 1. Add a CLI: read URLs from stdin (one per line) instead of the hardcoded
//    list. Hint: `tokio::io::{AsyncBufReadExt, BufReader}` over `tokio::io::stdin()`.
//
// 2. Add retries: if `fetch_title` errors, retry up to 3 times with
//    exponential backoff (e.g. 200ms, 400ms, 800ms).
//
// 3. Replace the `Vec<JoinHandle<...>>` collection pattern with
//    `futures::stream::iter(urls).map(...).buffer_unordered(MAX_INFLIGHT)`.
//    Add `futures = "0.3"` to Cargo.toml. Notice it eliminates the explicit
//    Semaphore.
//
// 4. (Stretch) Wrap the whole pipeline in a `tokio::select!` that races it
//    against a 30-second timeout. On timeout, print partial results.
//
// 5. (Big stretch) Add a tiny axum web server that exposes `GET /fetch?url=...`,
//    runs the pipeline for one URL, and returns JSON `{title, ms}`.

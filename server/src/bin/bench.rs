use std::{
    sync::{ 
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    time::{ Instant, timeout },
};

const CONCURRENT_CLIENTS: usize = 2000;
const MOVES_PER_CLIENT: usize = 20;

#[derive(Default)]
struct Metrics {
    connected: AtomicUsize,
    games_started: AtomicUsize,
    moves_processed: AtomicUsize,
    errors: AtomicUsize,
}

#[tokio::main]
async fn main() {
    println!("Starting TCP Connection Storm...");
    println!("Target: {} concurrent clients.", CONCURRENT_CLIENTS);

    let metrics = Arc::new(Metrics::default());
    let mut join_set = tokio::task::JoinSet::new();
    let start_time = Instant::now();

    for id in 0..CONCURRENT_CLIENTS {
        let metrics_clone = Arc::clone(&metrics);
        join_set.spawn(async move {
            run_simulated_client(id, metrics_clone).await;
        });
    }

    while let Some(_) = join_set.join_next().await {}

    let elapsed = start_time.elapsed();
    let elapsed_oi = if elapsed > Duration::from_secs(10) { elapsed - Duration::from_secs(10) } else { elapsed }; // Orphans Ignored

    println!("\n===== BENCHMARK COMPLETE =====");
    println!("Total Time:               {:.2?}", elapsed);
    println!("Time (Ignoring Orphans):  {:.2?}", elapsed_oi);
    println!("Clients Connected:        {}/{}", metrics.connected.load(Ordering::Relaxed), CONCURRENT_CLIENTS);
    println!("Games Started:            {}/{}", metrics.games_started.load(Ordering::Relaxed) / 2, CONCURRENT_CLIENTS/2);
    println!("Total Server Replies:     {}", metrics.moves_processed.load(Ordering::Relaxed));
    println!("Connection Errors:        {}", metrics.errors.load(Ordering::Relaxed));
    let total_messages = metrics.moves_processed.load(Ordering::Relaxed);
    let rps = (total_messages as f64) / elapsed_oi.as_secs_f64();
    println!("Throughput:               {:.2} messages/sec", rps);
}

async fn run_simulated_client(_id: usize, metrics: Arc<Metrics>) {
    let connect_future = TcpStream::connect("127.0.0.1:8080");

    // Timeout the initial connection phase to avoid hanging clients
    let mut stream = match timeout(Duration::from_secs(5), connect_future).await {
        Ok(Ok(s)) => s,
        _ => {
            metrics.errors.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };

    metrics.connected.fetch_add(1, Ordering::Relaxed);

    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader).lines();

    // Timeout the Matchmaking phas
    // If they get orphaned in the queue, they will give up and exit.
    let queue_future = reader.next_line();
    if let Ok(Ok(Some(line))) = timeout(Duration::from_secs(10), queue_future).await {
        if line.starts_with("s:") {
            metrics.games_started.fetch_add(1, Ordering::Relaxed);

            // Spam moves
            for _ in 0..MOVES_PER_CLIENT {
                if writer.write_all(b"m:12:28\n").await.is_err() {
                    break;
                }

                // Timeout waiting for move validation replies
                if let Ok(Ok(Some(_reply))) = timeout(Duration::from_secs(2), reader.next_line()).await {
                    metrics.moves_processed.fetch_add(1, Ordering::Relaxed);
                } else {
                    break; 
                }
            }
        }
    }

    let _ = writer.write_all(b"q\n").await;
}

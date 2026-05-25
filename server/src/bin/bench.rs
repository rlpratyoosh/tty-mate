use std::{
    sync::{ 
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
    env,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    time::{ Instant, timeout },
};

#[derive(Default)]
struct Metrics {
    connected: AtomicUsize,
    games_started: AtomicUsize,
    moves_processed: AtomicUsize,
    errors: AtomicUsize,
}

#[tokio::main]
async fn main() {
    let arguements: Vec<String> = env::args().collect();

    let mut concurrent_clients: usize = 2000;
    let mut moves_per_client: usize = 20;
    let mut timeout_time: u64 = 10;

    let mut i = 1;
    while i < arguements.len() {
        match arguements[i].as_str() {
            "-c" => {
                if let Some(val) = arguements.get(i + 1) {
                    concurrent_clients = val.parse().unwrap_or(concurrent_clients);
                }
                i += 1;
            },
            "-m" => {
                if let Some(val) = arguements.get(i + 1) {
                    moves_per_client = val.parse().unwrap_or(moves_per_client);
                }
                i += 1;
            },
            "-t" => {
                if let Some(val) = arguements.get(i + 1) {
                    timeout_time = val.parse().unwrap_or(timeout_time);
                }
                i += 1;
            },
            _ => {
                println!("Unknown argument: {}, Falling back to default!", arguements[i]);
            },
        }
        i += 1;
    }

    println!("Starting TCP Connection Storm...");
    println!("Target: {} concurrent clients.", concurrent_clients);

    let metrics = Arc::new(Metrics::default());
    let mut join_set = tokio::task::JoinSet::new();
    let start_time = Instant::now();

    for id in 0..concurrent_clients {
        let metrics_clone = Arc::clone(&metrics);
        join_set.spawn(async move {
            run_simulated_client(id, metrics_clone, timeout_time, moves_per_client).await;
        });
    }

    while let Some(_) = join_set.join_next().await {}

    let elapsed = start_time.elapsed();
    let elapsed_oi = if elapsed > Duration::from_secs(timeout_time) { elapsed - Duration::from_secs(timeout_time) } else { elapsed }; // Orphans Ignored

    println!("\n===== BENCHMARK COMPLETE =====");
    println!("Total Time:               {:.2?}", elapsed);
    println!("Time (Ignoring Orphans):  {:.2?}", elapsed_oi);
    println!("Clients Connected:        {}/{}", metrics.connected.load(Ordering::Relaxed), concurrent_clients);
    println!("Games Started:            {}/{}", metrics.games_started.load(Ordering::Relaxed) / 2, concurrent_clients/2);
    println!("Total Server Replies:     {}", metrics.moves_processed.load(Ordering::Relaxed));
    println!("Connection Errors:        {}", metrics.errors.load(Ordering::Relaxed));
    let total_messages = metrics.moves_processed.load(Ordering::Relaxed);
    let rps = (total_messages as f64) / elapsed_oi.as_secs_f64();
    println!("Throughput:               {:.2} messages/sec", rps);
}

async fn run_simulated_client(_id: usize, metrics: Arc<Metrics>, timeout_time: u64, moves_per_client: usize) {
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

    // Timeout the Matchmaking phase
    // If they get orphaned in the queue, they will give up and exit.
    let queue_future = reader.next_line();
    if let Ok(Ok(Some(line))) = timeout(Duration::from_secs(timeout_time), queue_future).await {
        if line.starts_with("s:") {
            metrics.games_started.fetch_add(1, Ordering::Relaxed);

            // Spam moves
            for _ in 0..moves_per_client {
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

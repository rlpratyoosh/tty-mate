use std::{
    io,
    sync::Arc,
    collections::{HashMap, VecDeque},
};
use tokio::{
    sync::Mutex,
    net::{TcpListener},
    task,
};
use tty_mate_server::{handle_client, Server};


#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let server = Arc::new(Mutex::new(
        Server {
            next_player_id: 0,
            next_game_id: 0,
            matchmaking_queue: VecDeque::new(),
            active_games: HashMap::new(),
        }
    ));

    loop {
        let (socket, _) = listener.accept().await?;
        let server = Arc::clone(&server);
        task::spawn(async move {
            handle_client(server, socket).await;
        });
    }
}


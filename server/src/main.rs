use tty_mate_core::board::{Board, PieceType, PieceColor};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::io;
use tokio::sync::{mpsc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;
use tty_mate_server::{handle_client, Server, Game, Player};


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


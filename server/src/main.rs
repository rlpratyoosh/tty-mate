use tty_mate_core::board::Board;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::io;
use tokio::sync::{mpsc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;

struct Player {
    id: usize,
    tx: mpsc::UnboundedSender<String>,
}

struct Game {
    id: usize,
    board: Board,
    white: Player,
    black: Player,
}

struct Server {
    next_player_id: usize,
    next_game_id: usize,
    matchmaking_queue: VecDeque<Player>,
    active_games: HashMap<usize, Arc<Mutex<Game>>>,
}

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

async fn handle_client(server: Arc<Mutex<Server>>, mut socket: TcpStream) {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let player_id: usize;
    let mut game_id: Option<usize> = None;

    {
        let mut server = server.lock().await;
        player_id = server.next_player_id;
        server.next_player_id += 1;

        let player = Player {
            id: player_id,
            tx,
        };

        if let Some(opponent) = server.matchmaking_queue.pop_front() {
            game_id = Some(server.next_game_id);
            server.next_game_id += 1;

            let game = Game {
                id: game_id.unwrap(),
                board: Board::new(),
                white: opponent,
                black: player,
            };

            server.active_games.insert(game_id.unwrap(), Arc::new(Mutex::new(game)));
        } else {
            server.matchmaking_queue.push_back(player);
        }
    }

    let (tcp_reader, tcp_writer) = socket.split();
    let mut tcp_reader = BufReader::new(tcp_reader).lines();

    loop {
        tokio::select! {
            message = tcp_reader.next_line() => {
                unimplemented!();
            }

            Some(message) = rx.recv() => {
                unimplemented!();
            }
        }
    }
}

//! # TTY Mate Server
//!
//! This library provides essential functionality for a broker-server for the TTY Mate chess application.
//! It manages TCP connections, player matchmaking, game state, and move validation. The server is
//! designed to handle multiple concurrent games and clients using Tokio's asynchronous runtime.

mod log;

use tty_mate_core::{Board, PieceColor};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    io,
};
use tokio::{
    sync::{mpsc, Mutex},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpStream, TcpListener},
    task,
};
use tty_mate_api::{
    ClientMessage,
    ServerMessage,
    GameError,
};
use log::Log;

pub struct Player {
    id: usize,
    tx: mpsc::UnboundedSender<ServerMessage>,
}

pub struct Game {
    id: usize,
    board: Board,
    white: Player,
    black: Player,
}

pub struct Server {
    pub next_player_id: usize,
    pub next_game_id: usize,
    pub matchmaking_queue: VecDeque<Player>,
    pub active_games: HashMap<usize, Arc<Mutex<Game>>>,
}

/// The main server loop that listens for incoming TCP connections.
///
/// This function takes in the TcpListener and initializes the shared server state wrapped in an
/// `Arc<Mutex<Server>>`. It then enters an infinite loop accepting new connections. For each
/// accepted connection, it clones the `Arc` to the server state and spawns a new Tokio task to
/// handle the client using the `handle_client` function.
pub async fn run_server(listener: TcpListener) -> io::Result<()> {
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
        }); // Spawn tasks per client
    }
}

/// Manages the full lifecycle of a single TCP client connection.
///
/// This function is spawned as an independent Tokio task. It handles the
/// initial ID generation, player matchmaking, and then enters an asynchronous
/// loop to manage `ClientMessage` and route `ServerMessage` payloads.
///
/// It strictly guarantees that all moves made are valid and upon a socket disconnect
/// or timeout, the player's internal state is cleanly popped from the global 
/// `Arc<Mutex<Server>>` to prevent memory leaks or orphaned locks.
pub async fn handle_client(server: Arc<Mutex<Server>>, mut socket: TcpStream) {
    let (tx, mut rx) = mpsc::unbounded_channel(); // Channel for inter-task communications, transmitting ServerMessage

    let player_id: usize;
    let mut game_id: Option<usize> = None;
    let mut game: Option<Arc<Mutex<Game>>> = None;
    let mut player_color: Option<PieceColor> = None;

    {
        let mut server = server.lock().await;
        player_id = server.next_player_id;
        server.next_player_id += 1;

        let player = Player {
            id: player_id,
            tx,
        };

        Log::info(&format!("Player {} connected", player_id));

        // If player exists in the queue, pop them and create a game, else add the player to the queue
        if let Some(opponent) = server.matchmaking_queue.pop_front() {
            let next_game_id = server.next_game_id;
            server.next_game_id += 1;

            let new_game = Game {
                id: next_game_id,
                board: Board::new(),
                white: opponent,
                black: player,
            };

            let new_game = Arc::new(Mutex::new(new_game));
            server.active_games.insert(next_game_id, Arc::clone(&new_game));

            let game_lock = new_game.lock().await;
            Log::info(format!("New game created with ID {} between Player {} (White) and Player {} (Black)", game_lock.id, game_lock.white.id, game_lock.black.id).as_str());

            // Send GameStart messages to both players
            let _ = game_lock.white.tx.send(ServerMessage::GameStart { game_id: next_game_id, color: PieceColor::White });
            let _ = game_lock.black.tx.send(ServerMessage::GameStart { game_id: next_game_id, color: PieceColor::Black });
        } else {
            server.matchmaking_queue.push_back(player);
            Log::info(&format!("Player {} added to matchmaking queue", player_id));
        }
    }

    let (tcp_reader, mut tcp_writer) = socket.split();
    let mut tcp_reader = BufReader::new(tcp_reader).lines();

    loop {
        tokio::select! {
            result = tcp_reader.next_line() => {
                let Ok(Some(line)) = result else {
                    Log::info(&format!("Player {} disconnected", player_id));
                    break;
                };

                let client_message = ClientMessage::parse(&line);

                match client_message {
                    Ok(ClientMessage::Move { from, to, piece_type }) => {
                        let game = match game.as_ref() {
                            Some(game) => game,
                            None => {
                                let msg = (GameError::NoGameFound).to_string();
                                Log::error(&format!("Player {} attempted to move without being in a game", player_id));
                                let _ = tcp_writer.write_all(msg.as_bytes()).await;
                                continue;
                            },
                        };
                        let mut game_lock = game.lock().await;

                        if player_color != Some(game_lock.board.get_current_turn()) {
                            let msg = (GameError::InvalidMove).to_string();
                            Log::error(&format!("Player {} attempted to move out of turn", player_id));
                            let _ = tcp_writer.write_all(msg.as_bytes()).await;
                            continue;
                        }

                        if let Err(_) = game_lock.board.move_piece(from, to) {
                            let msg = (GameError::InvalidMove).to_string();
                            Log::error(&format!("Player {} attempted an invalid move from {:?} to {:?}", player_id, from, to));
                            let _ = tcp_writer.write_all(msg.as_bytes()).await;
                            continue;
                        }
                        Log::info(&format!("Player {} moved from {:?} to {:?}", player_id, from, to));

                        // Notify opponent of the move
                        match game_lock.board.get_current_turn() {
                            PieceColor::White => {
                                let _ = game_lock.white.tx.send(ServerMessage::Move { from, to, piece_type });
                            },
                            PieceColor::Black => {
                                let _ = game_lock.black.tx.send(ServerMessage::Move { from, to, piece_type });
                            }
                        }
                    },
                    Ok(ClientMessage::Quit) => {
                        Log::info(&format!("Player {} quit the game", player_id));
                        break;
                    },
                    Err(e) => {
                        let _ = tcp_writer.write_all(e.to_string().as_bytes()).await;
                    }
                }
            }

            Some(message) = rx.recv() => {
                match message {
                    ServerMessage::GameStart { game_id: new_game_id, color: _ } => {
                        let server_lock = server.lock().await;

                        let try_game = server_lock.active_games.get(&new_game_id);
                        let new_game = match try_game.as_ref() {
                            Some(game) => game,
                            None => {
                                Log::error(&format!("Game with ID {} not found for Player {}", new_game_id, player_id));
                                continue;
                            }
                        };

                        game = Some(Arc::clone(&new_game));
                        game_id = Some(new_game_id);
                        player_color = if game.as_ref().unwrap().lock().await.white.id == player_id { // Safe to unwrap since we just created the game
                            Some(PieceColor::White)
                        } else {
                            Some(PieceColor::Black)
                        };

                        let message = (message).to_string();
                        let _ = tcp_writer.write_all(message.as_bytes()).await;
                    },
                    ServerMessage::Move { from: _, to: _, piece_type: _ } => {
                        let message = (message).to_string();
                        let _ = tcp_writer.write_all(message.as_bytes()).await;
                    },
                    ServerMessage::GameAborted => {
                        let message = (message).to_string();
                        let _ = tcp_writer.write_all(message.as_bytes()).await;
                    }
                }
            }
        }
    }

    Log::info(&format!("Cleaning up state for Player {}", player_id));
    let mut server_state = server.lock().await;

    // If game created, remove game, else remove player from the queue
    if let Some(active_game_id) = game_id {
        let game_lock = game.as_ref().unwrap().lock().await; // Safe to unwrap since we only set game_id if we successfully created a game
        match player_color { // Notify opponent of game abortion
            Some(PieceColor::Black) => { let _ = game_lock.white.tx.send(ServerMessage::GameAborted); },
            Some(PieceColor::White) => { let _ = game_lock.black.tx.send(ServerMessage::GameAborted); },
            _ => {},
        };
        if server_state.active_games.remove(&active_game_id).is_some() {
            Log::info(&format!("Game {} aborted due to Player {} disconnecting.", active_game_id, player_id));
        }
    } else {
        server_state.matchmaking_queue.pop_front(); 
        Log::info(&format!("Removed Player {} from the matchmaking queue.", player_id));
    }
}

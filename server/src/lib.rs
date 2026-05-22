use tty_mate_core::board::{Board, PieceType, PieceColor};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::io;
use tokio::sync::{mpsc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

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

enum ClientMessage {
    Move { from: usize, to: usize, piece_type: Option<PieceType>},
}

enum ServerMessage {
    GameStart { game_id: usize, color: PieceColor },
    Move { from: usize, to: usize, piece: PieceType },
}

pub async fn handle_client(server: Arc<Mutex<Server>>, mut socket: TcpStream) {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let player_id: usize;
    let mut game_id: Option<usize> = None;
    let mut game: Option<Arc<Mutex<Game>>> = None;

    {
        let mut server = server.lock().await;
        player_id = server.next_player_id;
        server.next_player_id += 1;

        let player = Player {
            id: player_id,
            tx,
        };

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
            game = Some(new_game);
            game_id = Some(next_game_id);

            let game_lock = game.as_ref().unwrap().lock().await;
            let _ = game_lock.white.tx.send(ServerMessage::GameStart { game_id: next_game_id, color: PieceColor::White });
        } else {
            server.matchmaking_queue.push_back(player);
        }
    }

    let (tcp_reader, mut tcp_writer) = socket.split();
    let mut tcp_reader = BufReader::new(tcp_reader).lines();

    loop {
        tokio::select! {
            result = tcp_reader.next_line() => {
                let Ok(Some(line)) = result else {
                    break;
                };
                let tokens: Vec<&str> = line.split(":").collect();
                let Some(mode) = tokens.get(0) else {
                    tcp_writer.write_all(b"e:Invalid command format\n").await.unwrap_or(());
                    continue;
                };
                let client_message = match *mode {
                    "m" => {
                        let Some(from) = tokens.get(1).and_then(|t| t.parse::<usize>().ok()) else { 
                            let _ = tcp_writer.write_all(b"e:Invalid command format\n").await;
                            continue; 
                        };
                        let Some(to) = tokens.get(2).and_then(|t| t.parse::<usize>().ok()) else { 
                            let _ = tcp_writer.write_all(b"e:Invalid command format\n").await;
                            continue; 
                        };
                        let piece_type: Option<PieceType> = match tokens.get(3) {
                            Some(char) => {
                                match char.to_lowercase().as_str() {
                                    "q" => Some(PieceType::Queen),
                                    "n" => Some(PieceType::Knight),
                                    "b" => Some(PieceType::Bishop),
                                    "r" => Some(PieceType::Rook),
                                    _ => {
                                        let _ = tcp_writer.write_all(b"e:Invalid command format\n").await;
                                        continue; 
                                    },
                                }
                            },
                            None => None,
                        };
                        ClientMessage::Move { from, to, piece_type }
                    },
                    _ => {
                        tcp_writer.write_all(b"e:Invalid command format\n").await.unwrap_or(());
                        continue;
                    },
                };

                match client_message {
                    ClientMessage::Move { from, to, piece_type } => {
                        let game = match game.as_ref() {
                            Some(game) => game,
                            None => {
                                tcp_writer.write_all(b"e:No game found\n").await.unwrap_or(());
                                continue;
                            }
                        };
                        let mut game_lock = game.lock().await;
                        if let Err(_) = game_lock.board.move_piece(from, to) {
                            tcp_writer.write_all(b"e:Invalid Move\n").await.unwrap_or(());
                        }
                    },
                }
            }

            Some(message) = rx.recv() => {
                match message {
                    ServerMessage::GameStart { game_id: new_game_id, color } => {
                        let server_lock = server.lock().await;

                        let try_game = server_lock.active_games.get(&new_game_id);
                        let new_game = match try_game.as_ref() {
                            Some(game) => game,
                            None => {
                                println!("Game not found");
                                continue;
                            }
                        };

                        game = Some(Arc::clone(&new_game));
                        game_id = Some(new_game_id);

                        let piece_color = if color == PieceColor::White { "w" } else { "b" };
                        let message = format!("g:{}:{}\n", new_game_id, piece_color);
                        let _ = tcp_writer.write_all(message.as_bytes()).await;
                    },
                    _ => {},
                }
            }
        }
    }
}

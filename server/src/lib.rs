use tty_mate_core::board::{Board, PieceColor};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::{
    sync::{mpsc, Mutex},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpStream},
};
use tty_mate_api::{
    ClientMessage,
    ServerMessage,
    GameError,
};

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

pub async fn handle_client(server: Arc<Mutex<Server>>, mut socket: TcpStream) {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let player_id: usize;
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

            let game_lock = new_game.lock().await;
            let _ = game_lock.white.tx.send(ServerMessage::GameStart { game_id: next_game_id, color: PieceColor::White });
            let _ = game_lock.black.tx.send(ServerMessage::GameStart { game_id: next_game_id, color: PieceColor::Black });
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

                let client_message = ClientMessage::parse(&line);

                match client_message {
                    Ok(ClientMessage::Move { from, to, piece_type }) => {
                        let game = match game.as_ref() {
                            Some(game) => game,
                            None => {
                                let msg = (GameError::NoGameFound).to_string();
                                let _ = tcp_writer.write_all(msg.as_bytes()).await;
                                continue;
                            },
                        };
                        let mut game_lock = game.lock().await;
                        if let Err(_) = game_lock.board.move_piece(from, to) {
                            let msg = (GameError::InvalidMove).to_string();
                            let _ = tcp_writer.write_all(msg.as_bytes()).await;
                            continue;
                        }
                        match game_lock.board.get_current_turn() {
                            PieceColor::White => {
                                let _ = game_lock.white.tx.send(ServerMessage::Move { from, to, piece_type });
                            },
                            PieceColor::Black => {
                                let _ = game_lock.black.tx.send(ServerMessage::Move { from, to, piece_type });
                            }
                        }
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
                                println!("Game not found");
                                continue;
                            }
                        };

                        game = Some(Arc::clone(&new_game));

                        let message = (message).to_string();
                        let _ = tcp_writer.write_all(message.as_bytes()).await;
                    },
                    ServerMessage::Move { from: _, to: _, piece_type: _ } => {
                        let message = (message).to_string();
                        let _ = tcp_writer.write_all(message.as_bytes()).await;
                    },
                }
            }
        }
    }
}

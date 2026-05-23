use tty_mate_core::board::{PieceColor, PieceType};

pub enum ClientMessage {
    Move { from: usize, to: usize, piece_type: Option<PieceType>},
    Quit,
}

pub enum ServerMessage {
    GameStart { game_id: usize, color: PieceColor },
    Move { from: usize, to: usize, piece_type: Option<PieceType> },
    GameAborted,
}

pub enum GameError {
    InvalidMessage,
    NoGameFound,
    InvalidMove,
}

impl GameError {
    pub fn parse(message: &str) -> Result<GameError, GameError> {
        let tokens: Vec<&str> = message.split(":").collect();
        let Some(mode) = tokens.get(0) else {
            return Err(GameError::InvalidMessage);
        };
        let game_error = match *mode {
            "e" => {
                let Some(error_code) = tokens.get(1) else { 
                    return Err(GameError::InvalidMessage);
                };
                match *error_code {
                    "i" => GameError::InvalidMessage,
                    "n" => GameError::NoGameFound,
                    "m" => GameError::InvalidMove,
                    _ => return Err(GameError::InvalidMessage),
                }
            },
            _ => return Err(GameError::InvalidMessage),
        };
        Ok(game_error)
    }

    pub fn to_string(&self) -> String {
        match self {
            GameError::InvalidMessage => "e:i\n".to_string(),
            GameError::NoGameFound => "e:n\n".to_string(),
            GameError::InvalidMove => "e:m\n".to_string(),
        }
    }
}

impl ServerMessage {
    pub fn parse(message: &str) -> Result<ServerMessage, GameError> {
        let tokens: Vec<&str> = message.split(":").collect();
        let Some(mode) = tokens.get(0) else {
            return Err(GameError::InvalidMessage);
        };
        let server_message = match *mode {
            "s" => {
                let Some(game_id) = tokens.get(1).and_then(|t| t.parse::<usize>().ok()) else { 
                    return Err(GameError::InvalidMessage);
                };
                let Some(color) = tokens.get(2).and_then(|t| {
                    match t.to_lowercase().as_str() {
                        "w" => Some(PieceColor::White),
                        "b" => Some(PieceColor::Black),
                        _ => None,
                    }
                }) else { 
                    return Err(GameError::InvalidMessage);
                };
                ServerMessage::GameStart { game_id, color }
            },
            "m" => {
                let Some(from) = tokens.get(1).and_then(|t| t.parse::<usize>().ok()) else { 
                    return Err(GameError::InvalidMessage);
                };
                let Some(to) = tokens.get(2).and_then(|t| t.parse::<usize>().ok()) else { 
                    return Err(GameError::InvalidMessage);
                };
                let piece_type: Option<PieceType> = match tokens.get(3) {
                    Some(char) => {
                        match char.to_lowercase().as_str() {
                            "q" => Some(PieceType::Queen),
                            "n" => Some(PieceType::Knight),
                            "b" => Some(PieceType::Bishop),
                            "r" => Some(PieceType::Rook),
                            _ => {
                                return Err(GameError::InvalidMessage);
                            },
                        }
                    },
                    None => None,
                };
                ServerMessage::Move { from, to, piece_type }
            },
            "a" => {
                ServerMessage::GameAborted
            },
            _ => {
                return Err(GameError::InvalidMessage);
            },
        };
        Ok(server_message)
    }

    pub fn to_string(&self) -> String {
        match self {
            ServerMessage::GameStart { game_id, color } => {
                let color_str = match color {
                    PieceColor::White => "w",
                    PieceColor::Black => "b",
                };
                format!("s:{}:{}\n", game_id, color_str)
            },
            ServerMessage::Move { from, to, piece_type } => {
                let piece_str = match piece_type {
                    Some(PieceType::Queen) => Some("q"),
                    Some(PieceType::Knight) => Some("n"),
                    Some(PieceType::Bishop) => Some("b"),
                    Some(PieceType::Rook) => Some("r"),
                    _ => None,
                };
                match piece_str {
                    Some(piece_str) => format!("m:{}:{}:{}\n", from, to, piece_str),
                    None => format!("m:{}:{}\n", from, to)
                }
            },
            ServerMessage::GameAborted => {
                "a\n".to_string()
            }
        }
    }
}

impl ClientMessage {
    pub fn parse(message: &str) -> Result<ClientMessage, GameError> {
        let tokens: Vec<&str> = message.split(":").collect();
        let Some(mode) = tokens.get(0) else {
            return Err(GameError::InvalidMessage);
        };
        let client_message = match *mode {
            "m" => {
                let Some(from) = tokens.get(1).and_then(|t| t.parse::<usize>().ok()) else { 
                    return Err(GameError::InvalidMessage);
                };
                let Some(to) = tokens.get(2).and_then(|t| t.parse::<usize>().ok()) else { 
                    return Err(GameError::InvalidMessage);
                };
                let piece_type: Option<PieceType> = match tokens.get(3) {
                    Some(char) => {
                        match char.to_lowercase().as_str() {
                            "q" => Some(PieceType::Queen),
                            "n" => Some(PieceType::Knight),
                            "b" => Some(PieceType::Bishop),
                            "r" => Some(PieceType::Rook),
                            _ => {
                                return Err(GameError::InvalidMessage);
                            },
                        }
                    },
                    None => None,
                };
                ClientMessage::Move { from, to, piece_type }
            },
            "q" => ClientMessage::Quit,
            _ => {
                return Err(GameError::InvalidMessage);
            },
        };
        Ok(client_message)
    }

    pub fn to_string(&self) -> String {
        match self {
            ClientMessage::Move { from, to, piece_type } => {
                let piece_str = match piece_type {
                    Some(PieceType::Queen) => Some("q"),
                    Some(PieceType::Knight) => Some("n"),
                    Some(PieceType::Bishop) => Some("b"),
                    Some(PieceType::Rook) => Some("r"),
                    _ => None,
                };
                match piece_str {
                    Some(piece_str) => format!("m:{}:{}:{}\n", from, to, piece_str),
                    None => format!("m:{}:{}\n", from, to)
                }
            },
            ClientMessage::Quit => {
                "q\n".to_string()
            },
        }
    }
}




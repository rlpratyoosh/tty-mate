//! # TTY-Mate API
//!
//! This library defines the client-server protocol for TTY-Mate, a terminal-based chess game.
//! It includes message formats for communication between the client and server, as well as error handling.

use tty_mate_core::{PieceColor, PieceType};

#[derive(Debug, PartialEq)]
pub enum ClientMessage {
    Move { from: usize, to: usize, piece_type: Option<PieceType>},
    Quit,
}

#[derive(Debug, PartialEq)]
pub enum ServerMessage {
    GameStart { game_id: usize, color: PieceColor },
    Move { from: usize, to: usize, piece_type: Option<PieceType> },
    GameAborted,
}

#[derive(Debug, PartialEq)]
pub enum GameError {
    InvalidMessage,
    NoGameFound,
    InvalidMove,
}

impl GameError {

    /// Parses a string message into a GameError. The expected format is `e:<error_code>`.
    /// <br><br>
    /// Error codes are:
    /// - i: InvalidMessage
    /// - n: NoGameFound
    /// - m: InvalidMove
    ///
    /// It returns `Ok(GameError::InvalidMessage)` if the given error message is "e:i" but
    /// `Err(GameError::InvalidMessage)` if it recieves an invalid message to parse.
    ///
    /// # Examples
    /// ```
    /// use tty_mate_api::GameError;
    ///
    /// let error = GameError::parse("e:n").unwrap();
    /// assert_eq!(error, GameError::NoGameFound)
    /// ```
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

    /// Converts a GameError into its string representation which can be used to transmit messages
    /// over TCP.
    /// 
    /// # Examples
    /// ```
    /// use tty_mate_api::GameError;
    ///
    /// let msg = (GameError::NoGameFound).to_string();
    /// assert_eq!(msg, "e:n\n".to_string());
    /// ```
    pub fn to_string(&self) -> String {
        match self {
            GameError::InvalidMessage => "e:i\n".to_string(),
            GameError::NoGameFound => "e:n\n".to_string(),
            GameError::InvalidMove => "e:m\n".to_string(),
        }
    }
}

impl ServerMessage {

    /// Parses a string message into a ServerMessage.
    /// <br><br>
    /// The expected formats are:
    /// - `s:<game_id>:<w|b>`: GameStart (e.g., "s:12:w" for game 12, playing as White)
    /// - `m:<from>:<to>:[piece_type]`: Move (e.g., "m:8:16" or "m:55:63:q" for queen promotion)
    /// - `a`: GameAborted
    ///
    /// It returns `Ok(ServerMessage)` on success, or `Err(GameError::InvalidMessage)`
    /// if it receives an invalid message payload to parse.
    ///
    /// # Examples
    /// ```
    /// use tty_mate_api::ServerMessage;
    /// use tty_mate_core::PieceColor;
    ///
    /// let msg = ServerMessage::parse("s:42:w").unwrap();
    /// assert_eq!(msg, ServerMessage::GameStart { game_id: 42, color: PieceColor::White });
    /// ```
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

    /// Converts a ServerMessage into its string representation which can be used to transmit messages
    /// over TCP.
    ///
    /// # Examples
    /// ```
    /// use tty_mate_api::ServerMessage;
    /// use tty_mate_core::PieceColor;
    ///
    /// let msg = (ServerMessage::GameStart { game_id: 42, color: PieceColor::White }).to_string();
    /// assert_eq!(msg, "s:42:w\n".to_string());
    /// ```
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

    /// Parses a string message into a ClientMessage.
    /// <br><br>
    /// The expected formats are:
    /// - `m:<from>:<to>:[piece_type]`: Move (e.g., "m:12:28" or "m:55:63:q" for queen promotion)
    /// - `q`: Quit
    ///
    /// It returns `Ok(ClientMessage)` on success, or `Err(GameError::InvalidMessage)`
    /// if it receives an invalid message payload to parse.
    ///
    /// # Examples
    /// ```
    /// use tty_mate_api::ClientMessage;
    ///
    /// let msg = ClientMessage::parse("q").unwrap();
    /// assert_eq!(msg, ClientMessage::Quit);
    /// ```
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

    /// Converts a ClientMessage into its string representation which can be used to transmit messages
    /// over TCP.
    /// 
    /// # Examples
    /// ```
    /// use tty_mate_api::ClientMessage;
    ///
    /// let msg = (ClientMessage::Quit).to_string();
    /// assert_eq!(msg, "q\n".to_string());
    /// ```
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


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn game_error_parse_valid() {
        assert_eq!(GameError::parse("e:i"), Ok(GameError::InvalidMessage));
        assert_eq!(GameError::parse("e:n"), Ok(GameError::NoGameFound));
        assert_eq!(GameError::parse("e:m"), Ok(GameError::InvalidMove));
    }

    #[test]
    fn game_error_parse_garbage() {
        assert_eq!(GameError::parse("e:x"), Err(GameError::InvalidMessage));
        assert_eq!(GameError::parse("x:i"), Err(GameError::InvalidMessage));
        assert_eq!(GameError::parse("ei"), Err(GameError::InvalidMessage));
        assert_eq!(GameError::parse(""), Err(GameError::InvalidMessage));
        assert_eq!(GameError::parse("e:i:extra:garbage"), Ok(GameError::InvalidMessage));
    }

    #[test]
    fn game_error_to_string() {
        assert_eq!(GameError::InvalidMessage.to_string(), "e:i\n");
        assert_eq!(GameError::NoGameFound.to_string(), "e:n\n");
        assert_eq!(GameError::InvalidMove.to_string(), "e:m\n");
    }


    #[test]
    fn server_message_parse_valid() {
        assert_eq!(ServerMessage::parse("s:42:w"), Ok(ServerMessage::GameStart { game_id: 42, color: PieceColor::White }));
        assert_eq!(ServerMessage::parse("s:0:b"), Ok(ServerMessage::GameStart { game_id: 0, color: PieceColor::Black }));

        assert_eq!(ServerMessage::parse("m:8:16"), Ok(ServerMessage::Move { from: 8, to: 16, piece_type: None }));

        assert_eq!(ServerMessage::parse("m:55:63:q"), Ok(ServerMessage::Move { from: 55, to: 63, piece_type: Some(PieceType::Queen) }));
        assert_eq!(ServerMessage::parse("m:55:63:n"), Ok(ServerMessage::Move { from: 55, to: 63, piece_type: Some(PieceType::Knight) }));
        assert_eq!(ServerMessage::parse("m:55:63:b"), Ok(ServerMessage::Move { from: 55, to: 63, piece_type: Some(PieceType::Bishop) }));
        assert_eq!(ServerMessage::parse("m:55:63:r"), Ok(ServerMessage::Move { from: 55, to: 63, piece_type: Some(PieceType::Rook) }));

        assert_eq!(ServerMessage::parse("a"), Ok(ServerMessage::GameAborted));
    }

    #[test]
    fn server_message_parse_garbage() {
        assert_eq!(ServerMessage::parse("s:42"), Err(GameError::InvalidMessage));
        assert_eq!(ServerMessage::parse("m:8"), Err(GameError::InvalidMessage));
        assert_eq!(ServerMessage::parse("s"), Err(GameError::InvalidMessage));
        assert_eq!(ServerMessage::parse("m"), Err(GameError::InvalidMessage));
        assert_eq!(ServerMessage::parse(""), Err(GameError::InvalidMessage));

        assert_eq!(ServerMessage::parse("s:abc:w"), Err(GameError::InvalidMessage));
        assert_eq!(ServerMessage::parse("s:42:x"), Err(GameError::InvalidMessage));
        assert_eq!(ServerMessage::parse("m:abc:16"), Err(GameError::InvalidMessage));
        assert_eq!(ServerMessage::parse("m:8:def"), Err(GameError::InvalidMessage));
        assert_eq!(ServerMessage::parse("m:55:63:x"), Err(GameError::InvalidMessage));

        assert_eq!(ServerMessage::parse("x:42:w"), Err(GameError::InvalidMessage));
    }

    #[test]
    fn server_message_to_string() {
        assert_eq!(ServerMessage::GameStart { game_id: 42, color: PieceColor::White }.to_string(), "s:42:w\n");
        assert_eq!(ServerMessage::GameStart { game_id: 0, color: PieceColor::Black }.to_string(), "s:0:b\n");
        assert_eq!(ServerMessage::Move { from: 8, to: 16, piece_type: None }.to_string(), "m:8:16\n");
        assert_eq!(ServerMessage::Move { from: 55, to: 63, piece_type: Some(PieceType::Queen) }.to_string(), "m:55:63:q\n");
        assert_eq!(ServerMessage::GameAborted.to_string(), "a\n");
    }


    #[test]
    fn client_message_parse_valid() {
        assert_eq!(ClientMessage::parse("m:12:28"), Ok(ClientMessage::Move { from: 12, to: 28, piece_type: None }));

        assert_eq!(ClientMessage::parse("m:55:63:q"), Ok(ClientMessage::Move { from: 55, to: 63, piece_type: Some(PieceType::Queen) }));

        assert_eq!(ClientMessage::parse("q"), Ok(ClientMessage::Quit));
    }

    #[test]
    fn client_message_parse_garbage() {
        assert_eq!(ClientMessage::parse("m:12"), Err(GameError::InvalidMessage));
        assert_eq!(ClientMessage::parse("m"), Err(GameError::InvalidMessage));
        assert_eq!(ClientMessage::parse(""), Err(GameError::InvalidMessage));

        assert_eq!(ClientMessage::parse("m:abc:28"), Err(GameError::InvalidMessage));
        assert_eq!(ClientMessage::parse("m:12:def"), Err(GameError::InvalidMessage));
        assert_eq!(ClientMessage::parse("m:55:63:x"), Err(GameError::InvalidMessage));

        assert_eq!(ClientMessage::parse("x:12:28"), Err(GameError::InvalidMessage));
    }

    #[test]
    fn client_message_to_string() {
        assert_eq!(ClientMessage::Move { from: 12, to: 28, piece_type: None }.to_string(), "m:12:28\n");
        assert_eq!(ClientMessage::Move { from: 55, to: 63, piece_type: Some(PieceType::Queen) }.to_string(), "m:55:63:q\n");
        assert_eq!(ClientMessage::Quit.to_string(), "q\n");
    }
}

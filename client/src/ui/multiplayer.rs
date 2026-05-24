use tty_mate_core::board::{Board, MoveList, PieceColor};
use tty_mate_api::{ServerMessage, ClientMessage, GameError};
use crate::ui::app::AppAction;
use crossterm::event::{KeyEvent, KeyCode};
use tokio::sync::mpsc;
use ratatui::{
    text::{Line, Span},
    style::{Style, Color},
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    buffer::Buffer,
    widgets::{Widget, Paragraph, Block},
};

pub struct MultiplayerGame {
    running: bool,
    hover: usize,
    selected: Option<usize>,
    current_move_list: MoveList,
    message: String,
    board: Board,
    move_history: Vec<String>,
    player_color: Option<PieceColor>,
    tx: mpsc::UnboundedSender<ClientMessage>,
}


impl MultiplayerGame {
    pub fn new(tx: mpsc::UnboundedSender<ClientMessage>) -> Self {
        MultiplayerGame {
            running: false,
            hover: 11, // Start with white's hover
            selected: None,
            current_move_list: MoveList::new(),
            message: "Welcome to TTY-Mate!".to_string(),
            board: Board::new(),
            move_history: Vec::new(),
            player_color: None,
            tx,
        }
    }

    pub fn handle_network_event(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::GameStart { game_id: _, color } => {
                self.running = true;
                self.player_color = Some(color);
                match color {
                    PieceColor::White => self.hover = 11, // Start with white's hover
                    PieceColor::Black => self.hover = 52, // Start wit hblack's hover
                }
                let color_str = match color {
                    PieceColor::White => "White",
                    PieceColor::Black => "Black",
                };
                self.message = format!("Game started! You are playing as {}.", color_str);
            },
            ServerMessage::Move { from, to, piece_type: _ } => {
                let _ = self.board.move_piece(from, to);
                let (from_r, from_c) = Board::index_to_coordinates(from);
                let (to_r, to_c) = Board::index_to_coordinates(to);
                let move_record = format!("{}{} to {}{}",
                    (b'a' + from_c as u8) as char, from_r+1,
                    (b'a' + to_c as u8) as char, to_r+1,
                );

                let color_prefix = if self.player_color != Some(PieceColor::White) { "White:" } else { "Black:" };
                self.move_history.push(format!("{} {}", color_prefix, move_record));


                let (is_game_over, winner) = self.board.is_game_over();
                if is_game_over {
                    if let Some(winner) = winner {
                        self.message = if winner == self.player_color.unwrap_or(PieceColor::White) {
                            "CHECKMATE! You win!".to_string()
                        } else {
                            "CHECKMATE! You lose!".to_string()
                        };
                    } else {
                        self.message = "STALEMATE! It's a draw!".to_string();
                    }
                    self.running = false;
                }
                return;
            },
            ServerMessage::GameAborted =>{
                self.running = false;
                let (is_game_over, _) = self.board.is_game_over(); 
                if is_game_over {
                    return;
                }
                self.message = "Game aborted by opponent!".to_string();
            },
        }
    }

    pub fn handle_network_error(&mut self, error: GameError) {
        match error {
            GameError::InvalidMove => {
                self.message = "Invalid move!".to_string();
            },
            GameError::NoGameFound => {
                self.message = "Keep patience!, still finding game for you!".to_string();
            },
            GameError::InvalidMessage => {
                self.message = "Invalid message.".to_string();
            },
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> AppAction {
        match key_event.code {
            KeyCode::Char('q') => {
                let _ = self.tx.send(ClientMessage::Quit);
                return AppAction::QuitToMenu; 
            },
            _ if !self.running => return AppAction::None,
            KeyCode::Esc => self.reset_moves(),
            KeyCode::Enter => self.handle_enter(),
            KeyCode::Char('h') | KeyCode::Left => self.hover_left(),
            KeyCode::Char('l') | KeyCode::Right => self.hover_right(),
            KeyCode::Char('j') | KeyCode::Down => self.hover_down(),
            KeyCode::Char('k') | KeyCode::Up => self.hover_up(),
            _ => {},
        }
        AppAction::None
    }

    fn handle_enter(&mut self) {
        let (is_game_over, _) = self.board.is_game_over();
        if is_game_over || !self.running { return; }

        if Some(self.board.get_current_turn()) != self.player_color {
            self.message = "Please wait for your opponent to move.".to_string();
            return;
        }

        if let Some(selected) = self.selected {
            if self.current_move_list.moves[0..self.current_move_list.count].contains(&self.hover) {
                if let Err(e) = self.board.move_piece(selected, self.hover) {
                    self.message = format!("Error: {}", e);
                    self.reset_moves();
                    return;
                }

                let (from_r, from_c) = Board::index_to_coordinates(selected);
                let (to_r, to_c) = Board::index_to_coordinates(self.hover);
                let move_record = format!("{}{} to {}{}",
                    (b'a' + from_c as u8) as char, from_r+1,
                    (b'a' + to_c as u8) as char, to_r+1,
                );

                let color_prefix = if self.player_color == Some(PieceColor::White) { "White:" } else { "Black:" };
                self.move_history.push(format!("{} {}", color_prefix, move_record));
                self.reset_moves();

                let _ = self.tx.send(ClientMessage::Move { 
                    from: selected,
                    to: self.hover,
                    piece_type: None
                });

                let (is_game_over, winner) = self.board.is_game_over();
                if is_game_over {
                    if let Some(winner) = winner {
                        self.message = if winner == self.player_color.unwrap_or(PieceColor::White) {
                            "CHECKMATE! You win!".to_string()
                        } else {
                            "CHECKMATE! You lose!".to_string()
                        };
                    } else {
                        self.message = "STALEMATE! It's a draw!".to_string();
                    }
                    self.running = false;
                }
                return;
            }
        }
        self.select();
    }

    fn select(&mut self) {
        let is_white_piece = self.board.get_piece_color(self.hover).map_or(false, |c| c == PieceColor::White);
        let is_black_piece = self.board.get_piece_color(self.hover).map_or(false, |c| c == PieceColor::Black);

        let piece_belongs_to_me = match self.player_color {
            Some(PieceColor::White) => is_white_piece,
            Some(PieceColor::Black) => is_black_piece,
            None => false,
        };

        if !piece_belongs_to_me {
            self.message = "You can only select your own pieces!".to_string();
            return;
        }

        self.current_move_list = match self.board.get_valid_moves(self.hover) {
            Ok(move_list) => {
                self.selected = Some(self.hover);
                move_list
            }
            Err(e) => {
                self.message = format!("Error: {}", e);
                self.reset_moves();
                return;
            }
        };
    }

    fn reset_moves(&mut self) {
        self.selected = None;
        self.current_move_list = MoveList::new();
    }

    fn hover_up(&mut self) {
        let (row, col) = Board::index_to_coordinates(self.hover);
        let view_color = self.player_color.unwrap_or(PieceColor::White);
        let row = if view_color == PieceColor::White {
            if row == 7 { 0 } else { row+1 }
        } else {
            if row == 0 { 7 } else { row-1 }
        };
        self.hover = 8 * row + col;
    }
    fn hover_down(&mut self) {
        let (row, col) = Board::index_to_coordinates(self.hover);
        let view_color = self.player_color.unwrap_or(PieceColor::White);
        let row = if view_color == PieceColor::White {
            if row == 0 { 7 } else { row-1 }
        } else {
            if row == 7 { 0 } else { row+1 }
        };
        self.hover = 8 * row + col;
    }
    fn hover_right(&mut self) {
        let (row, col) = Board::index_to_coordinates(self.hover);
        let col = if col == 7 { 0 } else { col+1 };
        self.hover = 8 * row + col;
    }
    fn hover_left(&mut self) {
        let (row, col) = Board::index_to_coordinates(self.hover);
        let col = if col == 0 { 7 } else { col-1 };
        self.hover = 8 * row + col;
    }
}

impl Widget for &MultiplayerGame {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(28), Constraint::Min(0)])
            .split(area);

        let horizontal_center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(70), Constraint::Min(0)])
            .split(vertical_center[1]);

        let sandbox = horizontal_center[1];

        let playground = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(48),
                Constraint::Length(2),
                Constraint::Length(20)
            ])
            .split(sandbox);

        let board_col = playground[0];
        let history_col = playground[2];

        let board_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(24), Constraint::Length(2), Constraint::Length(2)])
            .split(board_col);

        let board_area = board_split[0];
        let msg_area = board_split[2];

        let cell_width = 6;
        let cell_height = 3;

        let view_color = self.player_color.unwrap_or(PieceColor::White);

        for r in 0..8 {
            for c in 0..8 {
                let light_color = (r+c) % 2 == 0;
                let idx = if view_color == PieceColor::White {
                    (8 * (7-r) + c) as usize
                } else {
                    (8 * r + c) as usize 
                };

                let is_our_turn = self.running && self.player_color == Some(self.board.get_current_turn());
                let mut bg_color = if idx == self.hover {
                    Color::Rgb(255, 180, 50)
                } else if is_our_turn && self.current_move_list.moves[0..self.current_move_list.count].contains(&idx) {
                    Color::Rgb(115, 130, 60)
                } else {
                    if light_color { Color::Rgb(235, 189, 140) } else { Color::Rgb(140, 75, 38) }
                };

                if let Some(selected) = self.selected {
                    if selected == idx { bg_color = Color::Rgb(200, 65, 45); }
                }

                if let Some(checked) = self.board.get_checked_index() {
                    if checked == idx { bg_color = Color::Rgb(180, 30, 30); }
                }

                let (display_char, fg_color) = match self.board.get_piece_char(idx) {
                    Some('p') => ('♟', Color::Black),
                    Some('r') => ('♜', Color::Black),
                    Some('n') => ('♞', Color::Black),
                    Some('b') => ('♝', Color::Black),
                    Some('q') => ('♛', Color::Black),
                    Some('k') => ('♚', Color::Black),
                    Some('P') => ('♙', Color::White),
                    Some('R') => ('♖', Color::White),
                    Some('N') => ('♘', Color::White),
                    Some('B') => ('♗', Color::White),
                    Some('Q') => ('♕', Color::White),
                    Some('K') => ('♔', Color::White),
                    _ => (' ', Color::Black),
                };

                let x = board_area.x + (c * cell_width);
                let y = board_area.y + (r * cell_height);

                for h in 0..cell_height {
                    let inner_y = y+h;
                    if x < board_area.right() && inner_y < board_area.bottom() {
                        let content = if h == cell_height/2 {
                            format!("  {}   ", display_char)
                        } else {
                            "      ".to_string()
                        };
                        buf.set_string(x, inner_y, content, Style::default().bg(bg_color).fg(fg_color));
                    }
                }
            }
        }

        let (is_game_over, _) = self.board.is_game_over();
        let display_msg = if !self.running || is_game_over || self.move_history.len() == 0 {
            self.message.as_str()
        } else {
            if Some(self.board.get_current_turn()) == self.player_color {
                "Your Turn"
            } else {
                "Waiting for opponent..."
            }
        };

        Paragraph::new(display_msg)
            .alignment(Alignment::Center)
            .style(Style::default().bold())
            .render(msg_area, buf);

        let history_text: Vec<Line> = self.move_history.iter()
            .map(|m| Line::from(Span::styled(m.as_str(), Style::default())))
            .collect();

        let text_height = self.move_history.len() as u16;
        let viewport_height = history_col.height;
        let display_scroll = text_height.saturating_sub(viewport_height);

        Paragraph::new(history_text)
            .block(Block::default())
            .alignment(Alignment::Left)
            .scroll((display_scroll, 0))
            .render(history_col, buf);
    }
}

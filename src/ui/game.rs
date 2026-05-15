use crossterm::{
    event::{
        KeyCode,
        KeyEvent,
    }
};
use ratatui::{
    text::{Line, Span},
    style::{Style, Color},
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    buffer::Buffer,
    widgets::{Widget, Paragraph, Block},
    symbols::border,
};

use tty_mate::core::board::{Board, MoveList, PieceColor};
use crate::ui::app::{AppAction};

pub struct Game {
    hover: usize,
    selected: Option<usize>,
    current_move_list: MoveList,
    message: String,
    board: Board,
    move_history: Vec<String>,
    white_turn: bool,
}

impl Game {
    pub fn default() -> Self {
        Game {
            hover: 27,
            selected: None,
            current_move_list: MoveList::new(),
            message: "Welcome to TTY-Mate!".to_string(),
            board: Board::new(),
            move_history: Vec::new(),
            white_turn: true,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> AppAction {
        match key_event.code {
            KeyCode::Char('q') => return AppAction::QuitToMenu,
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
        if is_game_over { return; }
 
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
 
                let color_prefix = if self.white_turn { "White:" } else { "Black:" };
                self.move_history.push(format!("{} {}", color_prefix, move_record));

                self.white_turn = !self.white_turn;
                self.reset_moves();

                let (is_game_over, winner) = self.board.is_game_over();
                if is_game_over {
                    if let Some(winner) = winner {
                        match winner {
                            PieceColor::White => self.message = "CHECKMATE! White wins!".to_string(),
                            PieceColor::Black => self.message = "CHECKMATE! Black wins!".to_string(),
                        }
                    } else {
                        self.message = "STALEMATE! It's a draw!".to_string();
                    }
                }
                return;
            }
        }
        self.select();
    }

    fn reset_moves(&mut self) {
        self.selected = None;
        self.current_move_list = MoveList::new();
    }

    fn select(&mut self) {
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

    fn hover_up(&mut self) {
        let (row, col) = Board::index_to_coordinates(self.hover);
        let row = if row == 7 { 0 } else { row+1 };
        self.hover = 8 * row + col;
    }
    fn hover_down(&mut self) {
        let (row, col) = Board::index_to_coordinates(self.hover);
        let row = if row == 0 { 7 } else { row-1 };
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

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Game ")
            .border_set(border::THICK);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let vertical_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(28), Constraint::Min(0)])
            .split(inner_area);

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

        for r in 0..8 {
            for c in 0..8 {
                let light_color = (r+c) % 2 == 0;
                let idx = (8 * (7-r) + c) as usize;
                let mut bg_color = if idx == self.hover {
                    Color::Rgb(255, 180, 50)
                } else if self.current_move_list.moves[0..self.current_move_list.count].contains(&idx) {
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
        let display_msg = if is_game_over {
            self.message.as_str()
        } else {
            if self.white_turn { "White's Turn" } else { "Black's Turn" }
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

use crossterm::{
    event::{
        self,
        KeyCode,
        KeyEventKind,
        KeyEvent,
        Event,
    }
};
use ratatui::{
    DefaultTerminal,
    Frame,
    text::{Line},
    style::{Style, Stylize, Color},
    layout::{Rect},
    buffer::Buffer,
    widgets::{Block, Widget, Paragraph},
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
    exit: bool,
}

impl Game {
    pub fn default() -> Self {
        Game {
            hover: 27,
            selected: None,
            current_move_list: MoveList::new(),
            message: "Welcome to TTY-Mate!".to_string(),
            board: Board::new(),
            exit: false,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> AppAction {
        match key_event.code {
            KeyCode::Char('q') => { 
               return AppAction::QuitToMenu;
            }
            KeyCode::Char('j') => self.hover_down(),
            KeyCode::Char('k') => self.hover_up(),
            KeyCode::Char('h') => self.hover_left(),
            KeyCode::Char('l') => self.hover_right(),
            KeyCode::Enter => self.handle_enter(),
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
                let from_r = 8 - from_r - 1;
                let (to_r, to_c) = Board::index_to_coordinates(self.hover);
                let to_r = 8 - to_r - 1;
                self.message = format!("Moved from {}{} to {}{}", (b'a' + from_c as u8) as char, 8-from_r, (b'a' + to_c as u8) as char, 8-to_r);
                self.reset_moves();
                let (is_game_over, winner) = self.board.is_game_over();
                if is_game_over {
                    if let Some(winner) = winner {
                        match winner {
                            PieceColor::White => self.message = "Game Over! White wins!".to_string(),
                            PieceColor::Black => self.message = "Game Over! Black wins!".to_string(),
                        }
                    } else {
                        self.message = "Game Over! It's a draw!".to_string();
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
        let row = if row == 0 { 7 } else { row-1 };
        self.hover = 8 * row + col;
    }

    fn hover_down(&mut self) {
        let (row, col) = Board::index_to_coordinates(self.hover);
        let row = if row == 7 { 0 } else { row+1 };
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

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" TTY-Mate ".bold());
        let block = Block::bordered()
            .title(title)
            .border_set(border::THICK);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let cell_width = 6;
        let cell_height = 3;

        let margin_x = area.width/10;
        let margin_y = area.height/6;

        for r in 0..8 {
            for c in 0..8 {
                let light_color = (r+c) % 2 == 0;
                let idx = (8 * (7-r) + c) as usize;
                let mut bg_color = if idx == self.hover {
                    Color::Rgb(255, 180, 50) // Hover: Glowing Amber
                } else if self.current_move_list.moves[0..self.current_move_list.count].contains(&idx) {
                    Color::Rgb(115, 130, 60) // Possible Move: Faded Pine / Olive Green
                } else {
                    if light_color {
                        Color::Rgb(235, 189, 140) // Light Square: Soft Maple
                    } else {
                        Color::Rgb(140, 75, 38)   // Dark Square: Deep Chestnut
                    }
                };

                if let Some(selected) = self.selected {
                    if selected == idx {
                        bg_color = Color::Rgb(200, 65, 45); // Selected: Burnt Crimson / Autumn Leaf
                    }
                }

                if let Some(checked) = self.board.get_checked_index() {
                    if checked == idx {
                        bg_color = Color::Rgb(180, 30, 30); // Checked: Burning Ember
                    }
                }

                let x = margin_x + inner_area.x + (c * cell_width);
                let y = margin_y + inner_area.y + (r * cell_height);
                let display_char = match self.board.get_piece_char(idx) {
                    Some(char) => char,
                    None => ' ',
                };

                for h in 0..cell_height {
                    let inner_y = y+h;
                    if x < inner_area.right() && inner_y < inner_area.bottom() {
                        let content = if h == cell_height/2 {
                            format!("  {}   ", display_char)
                        } else {
                            "      ".to_string()
                        };
                        buf.set_string(x, inner_y, content, Style::default().bg(bg_color).fg(Color::Black));
                    }
                }
            }
        }

        let message_area = Rect {
            x: area.x,
            y: area.y + area.height - margin_y*3/2,
            width: area.width/2,
            height: 3,
        };

        Paragraph::new(self.message.as_str())
            .alignment(ratatui::layout::Alignment::Center)
            .render(message_area, buf);
    }
}

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
    widgets::{Block, Widget},
    symbols::border,
};

use tty_mate::Board;

fn main() -> std::io::Result<()> {
    ratatui::run(|mut terminal| App::default().run(terminal))
}

struct App {
    hover: usize,
    board: Board,
    exit: bool,
}

impl App {
    fn default() -> Self {
        App {
            hover: 27,
            board: Board::new(),
            exit: false,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.exit {
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, f: &mut Frame) {
        f.render_widget(self, f.area());
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => self.hover_down(),
            KeyCode::Char('k') => self.hover_up(),
            KeyCode::Char('h') => self.hover_left(),
            KeyCode::Char('l') => self.hover_right(),
            _ => {},
        }
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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" TTY-Mate ".bold());
        let block = Block::bordered()
            .title(title)
            .border_set(border::THICK);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let cell_width = 6;
        let cell_height = 3;

        let margin_x = 12;
        let margin_y = 9;

        for r in 0..8 {
            for c in 0..8 {
                let light_color = (r+c) % 2 == 0;
                let idx = (8 * (7-r) + c) as usize;
                let bg_color = if idx == self.hover {
                    Color::Rgb(100, 250, 200)
                } else {
                    if light_color {
                        Color::Rgb(200, 200, 200)
                    } else {
                        Color::Rgb(100, 100, 100)
                    }
                };
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
    }
}

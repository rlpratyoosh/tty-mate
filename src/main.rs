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
    board: Board,
    exit: bool,
}

impl App {
    fn default() -> Self {
        App {
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
            _ => {},
        }
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
                let bg_color = if light_color {
                    Color::Rgb(200, 200, 200)
                } else {
                    Color::Rgb(100, 100, 100)
                };
                let x = margin_x + inner_area.x + (c * cell_width);
                let y = margin_y + inner_area.y + (r * cell_height);
                let display_char = match self.board.get_piece_char(r.into(), c.into()) {
                    Some(char) => char,
                    None => ' ',
                };

                for h in 0..cell_height {
                    let inner_y = y+h;
                    if x < inner_area.right() && inner_y < inner_area.bottom() {
                        let spaces = " ".repeat((cell_width/2) as usize);
                        let content = if h == cell_height/2 {
                            format!("{}{}{}", spaces, display_char, spaces)
                        } else {
                            format!("{} {}", spaces, spaces)
                        };
                        buf.set_string(x, inner_y, content, Style::default().bg(bg_color).fg(Color::Black));
                    }
                }
            }
        }
    }
}

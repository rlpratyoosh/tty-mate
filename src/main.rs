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
    text::{Line, Text},
    style::Stylize,
    layout::Rect,
    buffer::Buffer,
    widgets::{Block, Paragraph, Widget},
    symbols::border,
};

fn main() -> std::io::Result<()> {
    ratatui::run(|mut terminal| App::default().run(terminal))
}

struct App {
    message: String,
    exit: bool,
}

impl App {
    fn default() -> Self {
        App {
            message: "Welcome to the world of chess!".to_string(),
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
            _ => self.change_message("Press q to quit!".to_string()),
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn change_message(&mut self, message: String) {
        self.message = message;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" TTY-Mate ".bold());
        let block = Block::bordered()
            .title(title)
            .border_set(border::THICK);
        let message = Text::from(self.message.as_str());
        Paragraph::new(message)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

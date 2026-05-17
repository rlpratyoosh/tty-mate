use crossterm::{
    event::{
        self,
        KeyEventKind,
        KeyEvent,
        Event,
    }
};
use ratatui::{
    DefaultTerminal,
    Frame,
};

use crate::ui::{
    game::Game,
    menu::Menu,
};

pub struct App {
    game: Game,
    menu: Menu,
    app_state: AppState,
    exit: bool,
}

pub enum AppState {
    Game,
    Menu,
}

pub enum AppAction {
    None,
    StartGame,
    QuitToMenu,
    Exit,
}

impl App {
    pub fn default() -> Self {
        App {
            game: Game::default(),
            menu: Menu::default(),
            app_state: AppState::Menu,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.exit {
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, f: &mut Frame) {
        match self.app_state {
            AppState::Menu => f.render_widget(&self.menu, f.area()),
            AppState::Game => f.render_widget(&self.game, f.area()),
        }
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
        let action = match self.app_state {
            AppState::Menu => self.menu.handle_key_event(key_event),
            AppState::Game => self.game.handle_key_event(key_event),
        };

        match action {
            AppAction::StartGame => { 
                self.game = Game::default();
                self.app_state = AppState::Game;
            }
            AppAction::QuitToMenu => self.app_state = AppState::Menu,
            AppAction::Exit => self.exit(),
            AppAction::None => {},
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}


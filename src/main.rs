pub mod ui;
use ui::game::Game;

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| Game::default().run(terminal))
}


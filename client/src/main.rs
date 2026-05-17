pub mod ui;
use ui::app::App;

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}


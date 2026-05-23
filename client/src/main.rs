pub mod ui;
use ui::app::App;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal).await;
    ratatui::restore();
    result
}


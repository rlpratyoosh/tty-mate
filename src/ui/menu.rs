use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
    symbols::border,
};

use crate::ui::app::AppAction;

pub struct Menu {
    hover: usize, // 0 for Start Local Game, 1 for Quit
}

impl Menu {
    pub fn default() -> Self {
        Menu { hover: 0 }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> AppAction {
        match key_event.code {
            KeyCode::Char('q') => return AppAction::Exit,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.hover < 1 {
                    self.hover += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.hover > 0 {
                    self.hover -= 1;
                }
            }
            KeyCode::Enter => {
                if self.hover == 0 {
                    return AppAction::StartGame;
                } else {
                    return AppAction::Exit;
                }
            }
            _ => {}
        }
        AppAction::None
    }
}

impl Widget for &Menu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Menu ")
            .border_set(border::THICK);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),       // Top spring
                Constraint::Length(7),    // ASCII Logo space
                Constraint::Length(2),    // Padding
                Constraint::Length(2),    // Start Button space
                Constraint::Length(2),    // Quit Button space
                Constraint::Min(0),       // Bottom spring
            ])
            .split(inner_area);

        let logo_lines = vec![
r#" _____ _____ ___  _      _      ____ _____ _____"#,
r#"/__ __Y__ __\\  \//     / \__/|/  _ Y__ __Y  __/"#,
r#"  / \   / \   \  /_____ | |\/||| / \| / \ |  \  "#,
r#"  | |   | |   / / \____\| |  ||| |-|| | | |  /_ "#,
r#"  \_/   \_/  /_/        \_/  \|\_/ \| \_/ \____\"#,
        ];

        let logo_text: Vec<Line> = logo_lines
            .into_iter()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Rgb(255, 180, 50)).bold())))
            .collect();

        Paragraph::new(logo_text)
            .alignment(Alignment::Center)
            .render(vertical_chunks[1], buf);

        let hover_bg = Color::Rgb(255, 180, 50); // Glowing Amber
        let hover_fg = Color::Black;
        let inactive_fg = Color::DarkGray;

        let start_text = if self.hover == 0 {
            Line::from(vec![Span::styled(" >> Start Local Game << ", Style::default().bg(hover_bg).fg(hover_fg))])
        } else {
            Line::from(vec![Span::styled("    Start Local Game    ", Style::default().fg(inactive_fg))])
        };

        let quit_text = if self.hover == 1 {
            Line::from(vec![Span::styled(" >> Quit << ", Style::default().bg(hover_bg).fg(hover_fg))])
        } else {
            Line::from(vec![Span::styled("    Quit    ", Style::default().fg(inactive_fg))])
        };

        Paragraph::new(start_text)
            .alignment(Alignment::Center)
            .render(vertical_chunks[3], buf);

        Paragraph::new(quit_text)
            .alignment(Alignment::Center)
            .render(vertical_chunks[4], buf);
    }
}

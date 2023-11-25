use crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::*;

use journal::*;
struct App {
    show_popup: bool,
}

impl App {
    fn new() -> App {
        App { show_popup: false }
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut journal = Journal::new(ui, update)?;
    journal.run().await?;
    let app = App::new();
    Ok(())
}

async fn update(event: Event) -> anyhow::Result<()> {
    if let Event::Key(key) = event {
        if key.kind == event::KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => anyhow::bail!("exited"),
                // KeyCode::Char('?') => app.show_popup = !app.show_popup,
                _ => {}
            }
        }
        // && key.code == KeyCode::Char('q') {
        //     anyhow::bail!("session finished");
        // }
    }

    Ok(())
}

fn ui(frame: &mut Frame<'_, '_>) {
    use ratatui::prelude::*;

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(20),
            Constraint::Length(1),
            Constraint::Percentage(60),
            Constraint::Length(1),
            Constraint::Percentage(20),
        ])
        .split(frame.size());
    frame.render_widget(
        Paragraph::new("Hello World!").block(Block::default().title("date").borders(Borders::ALL)),
        layout[0],
    );
    frame.render_widget(
        Paragraph::new("Hello World!")
            .block(Block::default().title("content").borders(Borders::ALL)),
        layout[2],
    );
    frame.render_widget(
        Paragraph::new("Hello World!").block(Block::default().title("tag").borders(Borders::ALL)),
        layout[4],
    );
}

use crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::*;

use journal::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut journal = Journal::new(ui, update)?;
    journal.run().await?;

    Ok(())
}

async fn update(event: Event) -> anyhow::Result<()> {
    if let Event::Key(key) = event {
        if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
            anyhow::bail!("session finished");
        }
    }

    Ok(())
}

fn ui(frame: &mut Frame<'_, '_>) {
    frame.render_widget(
        Paragraph::new("Hello World!")
            .block(Block::default().title("Greeting").borders(Borders::ALL)),
        frame.size(),
    );
}

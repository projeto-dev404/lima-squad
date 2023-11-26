use crossterm::event::{self, Event, KeyCode};
use journal::*;
use ratatui::widgets::*;

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    let mut journal = Journal::new(ui, update)?;
    journal.run().await?;
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
    let area = frame.size();
    let popup_area = Rect {
        x: area.width / 4,
        y: area.height / 3,
        width: area.width / 2,
        height: area.height / 3,
    };
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
    let popup = Popup::default()
        .content("Hello world!")
        .style(Style::new().yellow())
        .title("With Clear")
        .title_style(Style::new().white().bold())
        .border_style(Style::new().red());
    frame.render_widget(popup, popup_area);
}

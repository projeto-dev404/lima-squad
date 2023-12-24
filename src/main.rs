use crossterm::event::{self, Event, KeyCode};
use journal::*;
use ratatui::widgets::*;

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    Journal::new(ui, update).await?.run().await.map(|_| ())
}

async fn update(event: Event, ctx: Ctx) -> anyhow::Result<()> {
    if let Event::Key(key) = event {
        if key.kind == event::KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => anyhow::bail!("exited"),
                KeyCode::Char(' ') => {
                    let todos = sqlx::query!("select * from todos")
                        .fetch_all(&ctx.database)
                        .await?;

                    for todo in todos {
                        dbg!(todo);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn ui(_ctx: Ctx) -> Box<dyn Fn(&mut Frame<'_, '_>)> {
    use ratatui::prelude::*;

    Box::new(move |frame| {
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
            Paragraph::new("Hello World!")
                .block(Block::default().title("date").borders(Borders::ALL)),
            layout[0],
        );
        frame.render_widget(
            Paragraph::new("Hello World!")
                .block(Block::default().title("content").borders(Borders::ALL)),
            layout[2],
        );
        frame.render_widget(
            Paragraph::new("Hello World!")
                .block(Block::default().title("tag").borders(Borders::ALL)),
            layout[4],
        );

        frame.render_widget(
            widgets::Popup::default()
                .content("Hello world!")
                .style(Style::new().yellow())
                .title("With Clear")
                .title_style(Style::new().white().bold())
                .border_style(Style::new().red()),
            popup_area,
        );
    })
}

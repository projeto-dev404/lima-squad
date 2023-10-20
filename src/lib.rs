use std::{
    future::Future,
    io::{Stdout, StdoutLock},
};

use crossterm::{
    event::{Event, EventStream},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use futures_util::TryStreamExt;
use ratatui::{prelude::CrosstermBackend, Frame as TuiFrame, Terminal};

pub mod database;

pub type Backend<'a> = CrosstermBackend<StdoutLock<'a>>;
pub type Frame<'a, 'b> = TuiFrame<'a, Backend<'b>>;
type DrawHandler = fn(&mut Frame<'_, '_>) -> ();
type UpdateHandler<Ret> = fn(Event) -> Ret;

pub struct Journal<'a, UpdateRet>
where
    UpdateRet: Future<Output = anyhow::Result<()>>,
{
    terminal: Terminal<Backend<'a>>,
    stdout: Stdout,
    draw: DrawHandler,
    update: UpdateHandler<UpdateRet>,
}

impl<'a, UpdateRet> Journal<'a, UpdateRet>
where
    UpdateRet: Future<Output = anyhow::Result<()>>,
{
    pub fn new(draw: DrawHandler, update: UpdateHandler<UpdateRet>) -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout.lock()))?;

        Ok(Self {
            terminal,
            stdout,
            draw,
            update,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut event_stream = EventStream::new();
        let update = self.update;

        loop {
            self.terminal.draw(self.draw)?;

            while let Some(event) = event_stream.try_next().await? {
                update(event).await?;
            }
        }
    }
}

impl<'a, UpdateRet> Drop for Journal<'a, UpdateRet>
where
    UpdateRet: Future<Output = anyhow::Result<()>>,
{
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = self.stdout.execute(LeaveAlternateScreen);
    }
}

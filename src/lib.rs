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
use ratatui::{prelude::*, Frame as TuiFrame, Terminal};

pub mod widgets;

pub type Backend<'a> = CrosstermBackend<StdoutLock<'a>>;
pub type Frame<'a, 'b> = TuiFrame<'a, Backend<'b>>;
type DrawHandler = fn(Ctx) -> Box<dyn Fn(&mut Frame<'_, '_>)>;
type UpdateHandler<Ret> = fn(Event, Ctx) -> Ret;

pub struct Context {
    pub database: sqlx::Pool<sqlx::Sqlite>,
}
pub type Ctx = &'static Context;

pub struct Journal<'a, UpdateRet>
where
    UpdateRet: Future<Output = anyhow::Result<()>>,
{
    terminal: Terminal<Backend<'a>>,
    stdout: Stdout,
    draw: DrawHandler,
    update: UpdateHandler<UpdateRet>,
    ctx: Ctx,
}

impl<'a, UpdateRet> Journal<'a, UpdateRet>
where
    UpdateRet: Future<Output = anyhow::Result<()>>,
{
    pub async fn new(draw: DrawHandler, update: UpdateHandler<UpdateRet>) -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout.lock()))?;

        let database = sqlx::SqlitePool::connect(env!("DATABASE_URL")).await?;
        let context = Context { database };

        Ok(Self {
            terminal,
            stdout,
            draw,
            update,
            ctx: Box::leak(context.into()),
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut event_stream = EventStream::new();
        let update = self.update;
        let draw = (self.draw)(self.ctx);

        loop {
            self.terminal.draw(&draw)?;

            while let Some(event) = event_stream.try_next().await? {
                update(event, self.ctx).await?;
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

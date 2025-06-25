mod data;
mod app;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};
use ratatui::{
    prelude::CrosstermBackend,
    Terminal,
};
use std::io::stdout;

use crate::app::App;
use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    path: String,
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let args = Args::parse();
    let mut app = App::new(args.path)?;

    loop {
        terminal.draw(|frame| {
            app.draw(frame);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q'|'Q') => break,
                _ => app.update(key)
            }
        }
    }

    disable_raw_mode()?;
    std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

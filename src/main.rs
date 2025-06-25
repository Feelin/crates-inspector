mod data;
mod app;
mod ui;
mod error;
use anyhow::Result;

use ratatui::crossterm::{event::{self, Event, KeyCode}, terminal, ExecutableCommand};
use ratatui::prelude::*;
use ratatui::{
    prelude::CrosstermBackend,
    Terminal,
};
use std::io::stdout;
use std::panic;

use crate::app::App;
use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    path: String,

    #[arg(short, long)]
    license: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    // === Help Notices etc. ===
    if args.license {
        print_license();
        return Ok(());
    }

    terminal::enable_raw_mode()?;
    
    let mut stdout = stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
 
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

    terminal::disable_raw_mode()?;
    std::io::stdout().execute(terminal::LeaveAlternateScreen)?;
    Ok(())
}

/// Prints license information to the console.
fn print_license() {
    print!("Rucola is released under the GNU General Public License v3, available at <https://www.gnu.org/licenses/gpl-3.0>.

Copyright (C) 2024 Linus Mußmächer <linus.mussmaecher@gmail.com>

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
    ");
}


/// Ratatui boilerplate to set up panic hooks
fn init_hooks() -> error::Result<()> {
    // Get a default panic hook
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        // Just restore the terminal.
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
    Ok(())
}


/// Ratatui boilerplate to put the terminal into a TUI state
fn init_terminal() -> std::io::Result<Terminal<impl ratatui::backend::Backend>> {
    std::io::stdout().execute(terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;
    Ok(terminal)
}

/// Ratatui boilerplate to restore the terminal to a usable state after program exits (regularly or by panic)
fn restore_terminal() -> std::io::Result<()> {
    std::io::stdout().execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}


/// Draws nothing but a loading screen with an indexing message.
/// Temporary screen while the programm is indexing.
fn draw_loading_screen(
    terminal: &mut Terminal<impl ratatui::backend::Backend>,
    message: &str,
) -> error::Result<()> {
    // Draw 'loading' screen
    terminal.draw(|frame| {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(message).alignment(Alignment::Center),
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Fill(1),
            ])
                .split(frame.area())[1],
        );
    })?;
    Ok(())
}
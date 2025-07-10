mod data;
mod app;
mod ui;
mod error;

use clap::Parser;
use log::error;
use ratatui::crossterm::{event::{self, Event, KeyCode, KeyEventKind}, terminal, ExecutableCommand};
use ratatui::prelude::*;
use ratatui::{
    prelude::CrosstermBackend,
    Terminal,
};
use std::panic;

use crate::app::App;

// use std::fs::OpenOptions;
// use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    path: String,

    #[arg(short, long)]
    license: bool,
}

fn main() -> error::Result<()> {
   /* // Initialize logger
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("error.log")
        .expect("Failed to create log file");

    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .write_style(env_logger::WriteStyle::Always)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter(None, log::LevelFilter::Error)
        .init();*/
    
    let args = Args::parse();
    // === Help Notices etc. ===
    if args.license {
        print_license();
        return Ok(());
    }

    // Initialize hooks & terminal (ratatui boilerplate)
    init_hooks()?;
    let mut terminal = init_terminal()?;
    let (mut app, errors) = App::new(&args.path, |message| draw_loading_screen(&mut terminal, message));
    let mut current_error: Option<error::Errors> = errors.into_iter().next_back();
    
    loop {
        terminal.draw(|frame: &mut Frame| {
            let area = frame.area();
            let buf = frame.buffer_mut();

            // Make sure area is large enough or show error
            if area.width < 120 || area.height < 25 {
                // area too small and no error -> show area error
                current_error = Some(error::Errors::SmallArea);
            }

            let app_area = match &current_error {
                // If there is an error to be displayed
                Some(e) => {
                    // Separate the usual app area into a small bottom line for the area and a big area for what can be displayed of the app.
                    let areas =
                        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

                    // Render the error to the bottom.
                    Widget::render(e.to_ratatui(), areas[1], buf);

                    // Return the rest of the area for the app to render in.
                    areas[0]
                }
                // No error => App can render in the entire area.
                None => area,
            };

            Widget::render(ratatui::widgets::Clear, app_area, buf);

            // Draw the actual application
            app.draw(app_area, buf);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q'|'Q') => break,
                    _ => app.update(key)
                }
            }
        }
    }
    
    if let Some(error) = &current_error {
        error!("Error initializing app: {}", error);
    }
    restore_terminal()?;
    Ok(())
}

/// Prints license information to the console.
fn print_license() {
    print!("Rucola is released under the GNU General Public License v3, available at <https://www.gnu.org/licenses/gpl-3.0>.

Copyright (C) 2024 William Fu <williamgeeker@gmail.com>

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
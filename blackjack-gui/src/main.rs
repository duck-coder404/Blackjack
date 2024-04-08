use std::error::Error;
use std::io;
use std::io::Stdout;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyEvent};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::App;

pub mod app;
mod game;
mod input;
pub mod ui;

#[derive(Debug, Parser)]
#[command(author, about, version)]
pub struct AppConfiguration {
    /// time in ms between two ticks.
    #[arg(short, long, default_value_t = 1000)]
    tick_rate: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new();
    let tick_rate = Duration::from_secs(1);
    let result = run_app(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    println!("{app:#?}");
    if let Err(err) = result {
        println!("{err:#?}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::display(f, app))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent {
                kind: event::KeyEventKind::Press,
                code,
                ..
            }) = event::read()?
            {
                app.input(code);
                last_tick = Instant::now();
            }
        }
        if app.should_quit {
            break;
        }
        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }
    }
    Ok(())
}

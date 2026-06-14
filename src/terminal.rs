//! Terminal setup, event loop, autosave, and graceful shutdown.

use std::fs;
use std::io::{Stdout, stdout};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use signal_hook::flag;

use crate::app::App;
use crate::app::{MissingFilePolicy, Mode};
use crate::event::handle_key;
use crate::persist::expand_user_path;
use crate::ui;

const AUTOSAVE_DEBOUNCE: Duration = Duration::from_secs(2);
const SAVE_RETRY_BACKOFF: Duration = Duration::from_secs(30);
const MESSAGE_TTL: Duration = Duration::from_secs(8);
const POLL_IDLE: Duration = Duration::from_millis(250);
const POLL_DIRTY: Duration = Duration::from_millis(100);

struct ProjectLaunch {
    path: PathBuf,
    on_missing: MissingFilePolicy,
}

fn install_terminal_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        original(info);
    }));
}

fn default_project_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(std::env::temp_dir);
    let mut dir = base;
    dir.push("rust_tui");
    let _ = fs::create_dir_all(&dir);

    let tdl = dir.join("default.tdl");
    let legacy_json = dir.join("tasks.json");
    if !tdl.exists() && legacy_json.exists() {
        let _ = fs::copy(&legacy_json, &tdl);
    }

    tdl
}

fn resolve_launch() -> ProjectLaunch {
    let mut args = std::env::args().skip(1);
    let Some(arg) = args.next() else {
        return ProjectLaunch {
            path: default_project_path(),
            on_missing: MissingFilePolicy::SampleData,
        };
    };

    if matches!(arg.as_str(), "--help" | "-h") {
        eprintln!(
            "Usage: rust_tui [project.tdl]\n\n\
             With no argument, opens ~/.local/share/rust_tui/default.tdl\n\
             Pass a path to open a portable project file, e.g.:\n\
             rust_tui ~/projects/my-app.tdl"
        );
        std::process::exit(0);
    }

    ProjectLaunch {
        path: expand_user_path(&arg),
        on_missing: MissingFilePolicy::EmptyProject,
    }
}

pub fn run() -> Result<()> {
    color_eyre::install()?;
    install_terminal_panic_hook();

    let launch = resolve_launch();

    let shutdown = Arc::new(AtomicBool::new(false));
    flag::register(signal_hook::consts::SIGINT, Arc::clone(&shutdown))?;
    flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))?;

    let mut terminal = init_terminal()?;
    let mut app = match launch.on_missing {
        MissingFilePolicy::SampleData => App::new(launch.path),
        MissingFilePolicy::EmptyProject => App::new_portable(launch.path),
    };

    let result = run_app(&mut terminal, &mut app, &shutdown);
    if !app.save_on_exit() && app.persist.dirty {
        eprintln!(
            "Warning: could not save unsaved changes to {}",
            app.persist.data_path.display()
        );
    }
    restore_terminal()?;

    result
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        stdout(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}

fn maybe_autosave(app: &mut App) {
    if !app.persist.dirty {
        return;
    }
    let now = std::time::Instant::now();
    if let Some(failed_at) = app.persist.last_save_failed
        && now.duration_since(failed_at) < SAVE_RETRY_BACKOFF
    {
        return;
    }
    let idle_long_enough = app
        .persist
        .last_change
        .is_some_and(|t| now.duration_since(t) >= AUTOSAVE_DEBOUNCE);
    if idle_long_enough {
        app.save();
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    shutdown: &AtomicBool,
) -> Result<()> {
    let mut message_shown_at = std::time::Instant::now();
    let mut last_message = app.message.clone();

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        maybe_autosave(app);

        if app.message != last_message {
            last_message = app.message.clone();
            message_shown_at = std::time::Instant::now();
        } else if !app.message.is_empty()
            && message_shown_at.elapsed() >= MESSAGE_TTL
            && !app.message.contains("Welcome to Deep Todo")
        {
            app.message.clear();
            last_message.clear();
        }

        terminal.draw(|f| ui::draw(f, app))?;

        let poll = if app.persist.dirty {
            POLL_DIRTY
        } else {
            POLL_IDLE
        };
        if event::poll(poll)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press && handle_key(app, key) => {
                    break;
                }
                Event::Mouse(m) => {
                    if matches!(app.mode, Mode::Editing | Mode::Help) {
                        if matches!(m.kind, MouseEventKind::Down(_)) {
                            if app.mode == Mode::Editing {
                                app.cancel_edit();
                            } else {
                                app.mode = Mode::Normal;
                            }
                        }
                    } else {
                        match m.kind {
                            MouseEventKind::ScrollUp => app.move_selection(-2),
                            MouseEventKind::ScrollDown => app.move_selection(2),
                            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                                app.handle_tree_click(m.column, m.row, false);
                            }
                            MouseEventKind::Down(crossterm::event::MouseButton::Right) => {
                                app.handle_tree_click(m.column, m.row, true);
                            }
                            _ => {}
                        }
                    }
                }
                Event::Resize(_, _) => {
                    terminal.clear()?;
                    app.rebuild_visible();
                }
                _ => {}
            }
        }
    }
    Ok(())
}

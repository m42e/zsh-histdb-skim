extern crate skim;
mod environment;
mod focus;
mod history;
mod location;
mod query;
mod title;

use crate::environment::*;
use crate::focus::focus_dir;
use crate::focus::focus_session;
use crate::focus::get_focus_dir;
use crate::focus::get_focus_session;
use crate::focus::reset_focus_dir;
use crate::focus::reset_focus_session;
use crate::history::History;
use crate::location::Location;
use crate::query::build_query;
use crate::title::generate_title;

use rusqlite::{params_from_iter, Connection, OpenFlags, Result};
use skim::prelude::*;
use std::env;
use std::thread;

fn read_entries(location: &Location, grouped: bool, tx_item: SkimItemSender) {
    let conn = match Connection::open_with_flags(
        get_histdb_database(),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(conn) => conn,
        Err(error) => {
            let _ = tx_item.send(Arc::new(format!("Cannot open database: {}", error)));
            return;
        }
    };
    let query = build_query(location, grouped);
    let mut stmt = match conn.prepare(&query.sql) {
        Ok(stmt) => stmt,
        Err(error) => {
            let _ = tx_item.send(Arc::new(format!(
                "Cannot get result from database: {}",
                error
            )));
            return;
        }
    };

    let entries = match stmt.query_map(params_from_iter(query.params.iter()), |row| {
        let cmd: String = row.get("cmd")?;
        let command_end = cmd.len();
        Ok(History {
            id: row.get("id")?,
            cmd,
            start: row.get("start")?,
            exit_status: row.get("exit_status")?,
            duration: row.get("duration")?,
            count: row.get("count")?,
            session: row.get("session")?,
            host: row.get("host")?,
            dir: row.get("dir")?,
            searchrange: [(History::COMMAND_START, command_end + History::COMMAND_START)],
        })
    }) {
        Ok(entries) => entries,
        Err(error) => {
            let _ = tx_item.send(Arc::new(format!(
                "Cannot get result from database: {}",
                error
            )));
            return;
        }
    };

    for entry in entries {
        match entry {
            Ok(history) => {
                if tx_item.send(Arc::new(history)).is_err() {
                    break;
                }
            }
            Err(error) => {
                let _ = tx_item.send(Arc::new(format!(
                    "Cannot read result from database: {}",
                    error
                )));
                break;
            }
        }
    }
}

struct SelectionResult {
    selected_cmd: Option<String>,
    abort: bool,
}

fn get_starting_location() -> Location {
    let mut location = get_default_tab();
    if location == Location::Session && get_current_session_id() == "" {
        location = Location::Directory;
    }
    location
}

fn show_history(thequery: String) -> Result<String, String> {
    let mut location = get_starting_location();
    let mut grouped = true;
    let mut query = thequery;
    loop {
        let title = generate_title(&location);
        let window_option = get_preview_window_option();
        let color_options = get_color();

        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .multi(false)
            .reverse(true)
            .prompt(Some("history >>"))
            .query(Some(&query))
            .color(Some(&color_options))
            .bind(vec![
                "f1:abort",
                "f2:abort",
                "f3:abort",
                "f4:abort",
                "f5:abort",
                "f6:abort",
                "f7:abort",
                "ctrl-r:abort",
                "ctrl-u:half-page-up",
                "ctrl-d:half-page-down",
            ])
            .header(Some(&title))
            .preview(Some("")) // preview should be specified to enable preview window
            .preview_window(Some(&window_option)) // preview should be specified to enable preview window
            .nosort(get_nosort_option())
            .build()
            .unwrap();

        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

        let handle = thread::spawn(move || {
            read_entries(&location, grouped, tx_item);
        });

        let selected_items = Skim::run_with(&options, Some(rx_item));
        handle.join().unwrap();

        let selection_result = process_result(&selected_items, &mut location, &mut grouped);
        if selection_result.abort {
            return Err("Aborted".to_string());
        }
        if let Some(selected_cmd) = selection_result.selected_cmd {
            return Ok(selected_cmd);
        }
        query = selected_items.map_or_else(String::new, |selected| selected.query);
    }
}

fn selected_history(selected: &SkimOutput) -> Option<&History> {
    selected
        .selected_items
        .first()?
        .as_any()
        .downcast_ref::<History>()
}

fn process_result(
    selected_items: &Option<SkimOutput>,
    loc: &mut Location,
    grouped: &mut bool,
) -> SelectionResult {
    if let Some(sel) = selected_items.as_ref() {
        match sel.final_key {
            Key::ESC | Key::Ctrl('c') | Key::Ctrl('d') | Key::Ctrl('z') => {
                return SelectionResult {
                    selected_cmd: None,
                    abort: true,
                };
            }
            Key::Enter => match selected_history(sel) {
                Some(history) => {
                    return SelectionResult {
                        selected_cmd: Some(history.command().to_string()),
                        abort: false,
                    };
                }
                None => {
                    return SelectionResult {
                        selected_cmd: None,
                        abort: true,
                    };
                }
            },
            Key::F(1) => {
                *loc = Location::Session;
            }
            Key::F(2) => {
                *loc = Location::Directory;
            }
            Key::F(3) => {
                *loc = Location::Machine;
            }
            Key::F(4) => {
                *loc = Location::Everywhere;
            }
            Key::F(5) => {
                *grouped = !*grouped;
            }
            Key::F(6) => {
                if let Some(history) = selected_history(sel) {
                    if get_focus_session().is_none() {
                        focus_session(&history.session());
                    } else {
                        reset_focus_session();
                    }
                }
            }
            Key::F(7) => {
                if let Some(history) = selected_history(sel) {
                    if get_focus_dir().is_none() {
                        focus_dir(&history.dir());
                    } else {
                        reset_focus_dir();
                    }
                }
            }
            Key::Ctrl('r') => {
                *loc = match *loc {
                    Location::Session => Location::Directory,
                    Location::Directory => Location::Machine,
                    Location::Machine => Location::Everywhere,
                    Location::Everywhere => Location::Session,
                };
            }
            _ => (),
        };
        return SelectionResult {
            selected_cmd: None,
            abort: false,
        };
    } else {
        return SelectionResult {
            selected_cmd: None,
            abort: true,
        };
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let query = args.get(1).cloned().unwrap_or_default();

    if query == "--version" {
        println!("v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let result = show_history(query);
    if result.is_ok() {
        println!("{}", result.ok().unwrap());
    } else {
        eprintln!("{}", result.err().unwrap());
        std::process::exit(1);
    }

    Ok(())
}

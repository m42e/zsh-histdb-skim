extern crate skim;
mod environment;
mod history;
mod location;
mod query;
mod title;

use crate::environment::*;
use crate::history::History;
use crate::location::Location;
use crate::query::build_query_string;
use crate::title::generate_title;

use rusqlite::{Connection, OpenFlags, Result};
use skim::prelude::*;
use std::env;
use std::thread;

fn read_entries(location: &Location, grouped: bool, tx_item: SkimItemSender) {
    let conn_res =
        Connection::open_with_flags(get_histdb_database(), OpenFlags::SQLITE_OPEN_READ_ONLY);
    if conn_res.is_err() {
        let _ = tx_item.send(Arc::new("Cannot open database"));
        drop(tx_item);
        return;
    }
    let conn = conn_res.unwrap();
    let s = build_query_string(&location, grouped);

    let stmt_result = conn.prepare(&s);
    if stmt_result.is_err() {
        let _ = tx_item.send(Arc::new(format!(
            "Cannot get result from database {}",
            stmt_result.err().unwrap()
        )));
        drop(tx_item);
        return;
    }
    let mut stmt = stmt_result.unwrap();

    let cats = stmt.query_map([], |row| {
        let cmd: String = row.get("cmd")?;
        let commandend = cmd.len() as usize;
        Ok(History {
            id: row.get("id")?,
            cmd: cmd,
            start: row.get("start")?,
            exit_status: row.get("exit_status")?,
            duration: row.get("duration")?,
            count: row.get("count")?,
            session: row.get("session")?,
            host: row.get("host")?,
            dir: row.get("dir")?,
            searchrange: [(
                History::COMMAND_START,
                commandend + (History::COMMAND_START),
            )],
        })
    });
    for person in cats.unwrap() {
        if person.is_ok() {
            let x = person.unwrap();
            let _ = tx_item.send(Arc::new(x));
        }
    }
    drop(tx_item);
}

struct SelectionResult {
    selected_cmd: Option<String>,
    abort: bool,
}

fn get_starting_location() -> Location {
    let mut location = Location::Session;
    if get_current_session_id() == "" {
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

        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .multi(false)
            .reverse(true)
            .prompt(Some("history >>"))
            .query(Some(&query))
            .bind(vec![
                "f1:abort",
                "f2:abort",
                "f3:abort",
                "f4:abort",
                "f5:abort",
                "ctrl-r:abort",
                "ctrl-u:half-page-up",
                "ctrl-d:half-page-down",
            ])
            .header(Some(&title))
            .preview(Some("")) // preview should be specified to enable preview window
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
        if selection_result.selected_cmd.is_some() {
            return Ok(selection_result.selected_cmd.unwrap());
        }
        query = selected_items.unwrap().query;
    }
}

fn process_result(
    selected_items: &Option<SkimOutput>,
    loc: &mut Location,
    grouped: &mut bool,
) -> SelectionResult {
    if selected_items.is_some() {
        let sel = selected_items.as_ref().unwrap();
        match sel.final_key {
            Key::ESC | Key::Ctrl('c') | Key::Ctrl('d') | Key::Ctrl('z') => {
                return SelectionResult {
                    selected_cmd: None,
                    abort: true,
                };
            }
            Key::Enter => {
                return SelectionResult {
                    selected_cmd: Some(format!(
                        "{}",
                        ((*sel.selected_items[0]).as_any().downcast_ref::<History>())
                            .unwrap()
                            .command()
                    )),
                    abort: false,
                };
            }
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
    let _conn =
        Connection::open_with_flags(get_histdb_database(), OpenFlags::SQLITE_OPEN_READ_ONLY);

    let args: Vec<String> = env::args().collect();
    let query = |args: Vec<String>| -> String {
        if args.len() > 1 {
            return args[1].to_string();
        }
        return "".to_string();
    }(args);

    if query == "--version" {
        println!("v0.8.6");
        std::process::exit(1);
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

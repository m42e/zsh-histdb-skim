extern crate skim;
use tuikit::prelude::*;
use std::env;
use skim::prelude::*;
use std::io::Cursor;
use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::time::SystemTime;
use chrono::{NaiveDate, NaiveDateTime};

fn get_date_format() -> String {
    let key = "HISTDB_FZF_FORCE_DATE_FORMAT";
    let forced_dateformat = env::var(key).unwrap_or("non-us".to_string()).to_lowercase();

    if forced_dateformat == "us" {
        return "%m/%d/%Y".to_string();
    }
    else{
        return "%d/%m/%Y".to_string();
    }
}

#[derive(Debug)]
struct History {
    id: i64,
    cmd: String,
    start: u64,
    exit_status: Option<i64>,
    duration: Option<i64>,
    count: i64,
    session: i64,
    host: String,
    dir: String,
}

impl SkimItem for History {
    fn text(&self) -> Cow<str> {
        let now = SystemTime::now();
        let now_secs = now.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();

        let seconds_since_midnight = now_secs % (24*3600);
        let starttime = NaiveDateTime::from_timestamp(self.start as i64, 0);
        let mut dateinfo = String::from("");
        if self.start > (now_secs - seconds_since_midnight) {
            dateinfo.push_str(&format!("{}", starttime.format("%H:%M")));
        }else {
            dateinfo.push_str(&format!("{}", starttime.format(&get_date_format())));
        }

        let mut information = format!("{:10} {}",dateinfo, self.cmd);
        Cow::Owned(information)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let mut information = String::from(format!("\x1b[1mDetails for {}\x1b[0m\n\n", self.id));
        information.push_str(&format!("\x1b[1m{:20}\x1b[0m{}\n", "Runtime", self.duration.unwrap_or(-1)));
        information.push_str(&format!("\x1b[1m{:20}\x1b[0m{}\n", "Host", self.host));
        information.push_str(&format!("\x1b[1m{:20}\x1b[0m{}\n", "Directory", self.dir));
        information.push_str(&format!("\x1b[1m{:20}\x1b[0m{}\n", "Exit Status", self.exit_status.unwrap_or(-1)));
        information.push_str(&format!("\x1b[1m{:20}\x1b[0m{}\n", "Session", self.session));
        information.push_str(&format!("\x1b[1m{:20}\x1b[0m{}\n", "Start Time", self.start));
        information.push_str(&format!("\x1b[1m{:20}\x1b[0m\n\n{}", "Command", self.cmd));
        ItemPreview::AnsiText(information)
    }
}

fn main() -> Result<()> {
    let key = "HOME";
    let path = env::var(key).unwrap();
    let conn = Connection::open(path + "/.histdb/zsh-history.db")?;

    let args: Vec<String> = env::args().collect();
    let query = |args : Vec<String>| -> String {
        if args.len() > 1 {
            return args[1].to_string();
        }
        return "run".to_string();
    }(args);

    let s = build_query_string(Location::Session);
    println!("{}", s);

    let mut stmt = conn.prepare(
        &s
    )?;

      let cats = stmt.query_map(NO_PARAMS, |row| {
        Ok( History {
            id: row.get(0)?,
            cmd: row.get(1)?,
            start: row.get(2)?,
            exit_status: row.get(3)?,
            duration: row.get(4)?,
            count: row.get(5)?,
            session: row.get(6)?,
            host: row.get(7)?,
            dir: row.get(8)?,
        })
    })?;

    let title = format!("{}\n{}\n―――――――――――――――――――――――――","Hello", "F1: Session, F2: Directory, F3: Host, F4: Everywhere");

    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .multi(false)
        .reverse(true)
        .bind(vec!["f1:abort","f2:abort","f3:abort","f4:abort","ctrl-r:abort",])
        .header(Some(&title))
        .preview(Some("")) // preview should be specified to enable preview window
        .build()
        .unwrap();

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
    for person in cats {
        let _ = tx_item.send(Arc::new(person?));
    }
    drop(tx_item); // so that skim could know when to stop waiting for more items.

    let selected_items = Skim::run_with(&options, Some(rx_item))
        .map(|out| {
            (out.final_key, out.selected_items)
        });

    println!("{}", selected_items.unwrap().0 == Key::F(1));

        /*
    for item in selected_items.iter() {
        print!("{}{}", item.output(), "\n");
    }
    */

    Ok(())
}

#[derive(PartialEq)]
enum Location {
    Session,
    Directory,
    Machine,
    Everywhere
}

fn build_query_string(theloc: Location) -> String {

    let dateformat = get_date_format();

      let mut query=String::from(" select history.id, commands.argv, max(start_time) as max_start, exit_status, duration, count() as count, history.session, places.host, places.dir ");
      query.push_str(" from history ");
      query.push_str("
        left join commands on history.command_id = commands.id
        left join places on history.place_id = places.id");
        if theloc == Location::Session || theloc == Location::Directory || theloc == Location::Machine {
            query.push_str(" where ");
            if theloc == Location::Session {
                query.push_str("session in (");
                let key = "HISTDB_SESSION";
                let session_id = env::var(key).unwrap_or(String::from(""));
                query.push_str(&session_id);
                query.push_str(")");
            }
            else if theloc == Location::Directory {
                query.push_str(" (places.dir like '");
                let current_dir = env::current_dir().unwrap();
                let cdir_string = current_dir.to_str().unwrap();
                query.push_str(cdir_string);
                query.push_str("')");
            }
            if theloc != Location::Everywhere {
                query.push_str(" and places.host='");
                let host = env::var("HISTDB_HOST").unwrap_or(String::from(""));
                query.push_str(&host);
                query.push_str("'");
                query.push_str(" group by history.command_id, history.place_id");
            }
        }
        query.push_str("
      order by max_start desc");
    return query;
}

fn test() ->Result<()>{
    let conn = Connection::open("~/.histdb/zsh-history.db")?;



    let mut stmt = conn.prepare(
        "SELECT c.name, cc.name from cats c
         INNER JOIN cat_colors cc
         ON cc.id = c.color_id;",
    )?;



    Ok(())
}

pub fn skimmain() {
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .build()
        .unwrap();

    let input = "aaaaa\nbbbb\nccc".to_string();

    // `SkimItemReader` is a helper to turn any `BufRead` into a stream of `SkimItem`
    // `SkimItem` was implemented for `AsRef<str>` by default
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    // `run_with` would read and show items from the stream
    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    for item in selected_items.iter() {
        print!("{}{}", item.output(), "\n");
    }
}



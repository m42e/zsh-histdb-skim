extern crate skim;
use crate::environment::*;
use chrono::{DateTime, Datelike, Local, TimeZone};
use humantime::format_duration;
use skim::prelude::*;
use std::time::Duration;
use textwrap::fill;

fn get_epoch_start_of_day() -> u64 {
    let now = Local::now();
    Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .earliest()
        .unwrap_or(now)
        .timestamp()
        .max(0) as u64
}

pub struct History {
    pub id: i64,
    pub cmd: String,
    pub start: u64,
    pub exit_status: Option<i64>,
    pub duration: Option<i64>,
    pub count: i64,
    pub session: i64,
    pub host: String,
    pub dir: String,
    pub searchrange: [(usize, usize); 1],
}

impl History {
    pub const FORMAT_DATE_LENGTH: usize = 10;
    pub const COMMAND_START: usize = (History::FORMAT_DATE_LENGTH + 1);

    pub fn command(&self) -> &String {
        return &self.cmd;
    }
    pub fn session(&self) -> String {
        return self.session.to_string();
    }
    pub fn dir(&self) -> String {
        return self.dir.to_string();
    }
}

impl History {
    fn format_date(&self, full: bool) -> String {
        let starttime: DateTime<Local> = match Local.timestamp_opt(self.start as i64, 0).single() {
            Some(starttime) => starttime,
            None => return "<INVALID DATE>".to_string(),
        };
        if full {
            let mut dateinfo = String::from("");
            dateinfo.push_str(&get_date_format());
            dateinfo.push_str(" %H:%M");
            return format!("{}", starttime.format(&dateinfo));
        } else if self.start > get_epoch_start_of_day() {
            return format!("{}", starttime.format("%H:%M"));
        } else {
            return format!("{}", starttime.format(&get_date_format()));
        }
    }

    fn format_or_none(x: Option<i64>) -> String {
        x.map_or_else(
            || "\x1b[37;1m<NONE>\x1b[0m".to_string(),
            |value| value.to_string(),
        )
    }

    fn format_duration(&self) -> String {
        match self.duration {
            Some(seconds) if seconds >= 0 => {
                format_duration(Duration::from_secs(seconds as u64)).to_string()
            }
            Some(_) => History::format_or_none(self.duration),
            None => History::format_or_none(None),
        }
    }
}

impl SkimItem for History {
    fn text(&self) -> Cow<'_, str> {
        let information = format!("{:10} {}", self.format_date(false), self.cmd);
        Cow::Owned(information)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let mut information = String::from(format!("\x1b[1mDetails for {}\x1b[0m\n\n", self.id));

        let mut tformat = |name: &str, value: &str| {
            information.push_str(&format!("\x1b[1m{:20}\x1b[0m{}\n", name, value));
        };

        tformat("Runtime", &self.format_duration());
        tformat("Host", &self.host);
        tformat("Executed", &self.count.to_string());
        tformat("Directory", &self.dir);
        tformat("Exit Status", &History::format_or_none(self.exit_status));
        tformat("Session", &self.session.to_string());
        tformat("Start Time", &self.format_date(false));
        information.push_str(&format!(
            "\x1b[1mCommand\x1b[0m\n\n{}\n",
            &fill(&self.cmd, _context.width.max(1))
        ));
        ItemPreview::AnsiText(information)
    }

    fn get_matching_ranges(&self) -> Option<&[(usize, usize)]> {
        Some(&self.searchrange)
    }
}

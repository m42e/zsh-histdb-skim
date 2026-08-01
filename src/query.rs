use crate::environment::*;
use crate::focus::get_focus_dir;
use crate::focus::get_focus_session;
use crate::location::Location;
use rusqlite::types::Value;

pub struct HistoryQuery {
    pub sql: String,
    pub params: Vec<Value>,
}

pub fn build_query(theloc: &Location, grouped: bool) -> HistoryQuery {
    let current_session = get_current_session_id();
    let current_dir = get_current_dir();
    let current_host = get_current_host();
    let focus_session = get_focus_session();
    let focus_dir = get_focus_dir();

    build_query_with_context(
        theloc,
        grouped,
        &current_session,
        &current_dir,
        &current_host,
        focus_session.as_deref(),
        focus_dir.as_deref(),
    )
}

fn build_query_with_context(
    theloc: &Location,
    grouped: bool,
    current_session: &str,
    current_dir: &str,
    current_host: &str,
    focus_session: Option<&str>,
    focus_dir: Option<&str>,
) -> HistoryQuery {
    let mut query = String::from("select history.id as id, commands.argv as cmd,");
    if !grouped {
        query.push_str(" start_time")
    } else {
        query.push_str(" max(start_time)")
    }
    query.push_str(" as start, exit_status, duration,");
    if !grouped {
        query.push_str(" 1")
    } else {
        query.push_str(" count()")
    }
    query.push_str(" as count, history.session as session, places.host as host, places.dir as dir");
    query.push_str(" from history");
    query.push_str(" left join commands on history.command_id = commands.id");
    query.push_str(" left join places on history.place_id = places.id");

    let mut predicates = Vec::new();
    let mut params = Vec::new();
    match theloc {
        Location::Session => {
            let session = focus_session.unwrap_or(current_session);
            predicates.push("session == ?".to_string());
            params.push(session_parameter(session));
        }

        Location::Directory => {
            let directory = focus_dir.unwrap_or(current_dir);
            predicates.push("places.dir == ?".to_string());
            params.push(Value::Text(directory.to_string()));
        }

        _ => {}
    };
    match theloc {
        Location::Machine | Location::Everywhere => {
            if let Some(directory) = focus_dir {
                predicates.push("places.dir == ?".to_string());
                params.push(Value::Text(directory.to_string()));
            }
            if let Some(session) = focus_session {
                predicates.push("session == ?".to_string());
                params.push(session_parameter(session));
            }
        }

        _ => {}
    };
    match theloc {
        Location::Session | Location::Directory | Location::Machine => {
            predicates.push("places.host == ?".to_string());
            params.push(Value::Text(current_host.to_string()));
        }
        _ => {}
    };

    if !predicates.is_empty() {
        query.push_str(" where ");
        query.push_str(&predicates.join(" and "));
    }

    if grouped {
        query.push_str(" group by history.command_id, history.place_id");
    }
    query.push_str(" order by start desc");

    HistoryQuery { sql: query, params }
}

fn session_parameter(session: &str) -> Value {
    match session.parse::<i64>() {
        Ok(session) => Value::Integer(session),
        Err(_) => Value::Null,
    }
}

#[cfg(test)]
mod query {
    use super::*;
    use regex::Regex;

    #[test]
    fn has_select_fields() {
        for l in vec![
            Location::Session,
            Location::Directory,
            Location::Machine,
            Location::Everywhere,
        ] {
            let query = build_query(&l, true).sql;
            assert!(query.contains("history.id as id"));
            assert!(query.contains("exit_status"));
            assert!(query.contains("start"));
            assert!(query.contains("duration"));
            assert!(query.contains("count"));
            assert!(query.contains("history.session as session"));
            assert!(query.contains("places.dir"));
        }
    }

    #[test]
    fn contains_host() {
        let re_host = Regex::new(r"places\.host == \?").unwrap();
        for l in vec![Location::Session, Location::Directory, Location::Machine] {
            let query = build_query(&l, true).sql;
            assert!(re_host.is_match(&query));
        }
        let query = build_query(&Location::Everywhere, true).sql;
        assert!(!re_host.is_match(&query));
    }

    #[test]
    fn contains_grouping() {
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        for l in vec![
            Location::Session,
            Location::Directory,
            Location::Machine,
            Location::Everywhere,
        ] {
            let query = build_query(&l, true).sql;
            assert!(re_group.is_match(&query));
        }
    }

    #[test]
    fn contains_no_grouping_if_disabled() {
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        let re_only_group = Regex::new(r"group").unwrap();
        for l in vec![
            Location::Session,
            Location::Directory,
            Location::Machine,
            Location::Everywhere,
        ] {
            let query = build_query(&l, false).sql;
            assert!(!re_only_group.is_match(&query));
            assert!(!re_group.is_match(&query));
        }
    }

    #[test]
    fn for_session() {
        let query = build_query(&Location::Session, true).sql;
        let re_session = Regex::new(r"session == \? and").unwrap();
        let re_host = Regex::new(r"places\.host == \?").unwrap();
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        assert!(re_session.is_match(&query));
        assert!(re_host.is_match(&query));
        assert!(re_group.is_match(&query));
    }

    #[test]
    fn for_directory() {
        let query = build_query(&Location::Directory, false).sql;
        let re_directory = Regex::new(r"places\.dir == \? and").unwrap();
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        assert!(re_directory.is_match(&query));
        assert!(!re_group.is_match(&query));
    }

    #[test]
    fn for_machine() {
        let query = build_query(&Location::Machine, true).sql;
        let re_session = Regex::new(r"session == \? and").unwrap();
        let re_place = Regex::new(r"dir == \? and").unwrap();
        let re_host = Regex::new(r"places\.host == \?").unwrap();
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        assert!(!re_session.is_match(&query));
        assert!(!re_place.is_match(&query));
        assert!(re_host.is_match(&query));
        assert!(re_group.is_match(&query));
    }
    #[test]
    fn for_everywhere() {
        let query = build_query(&Location::Everywhere, true).sql;
        let re_session = Regex::new(r"session == \? and").unwrap();
        let re_place = Regex::new(r"dir == \? and").unwrap();
        let re_host = Regex::new(r"places\.host == \?").unwrap();
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        assert!(!re_session.is_match(&query));
        assert!(!re_place.is_match(&query));
        assert!(!re_host.is_match(&query));
        assert!(re_group.is_match(&query));
    }

    #[test]
    fn binds_filter_values_instead_of_interpolating_them() {
        let query = build_query_with_context(
            &Location::Directory,
            true,
            "4713",
            "/tmp/O'Brien",
            "host'with-quote",
            None,
            None,
        );

        assert!(query.sql.contains("places.dir == ? and places.host == ?"));
        assert!(!query.sql.contains("O'Brien"));
        assert_eq!(
            query.params,
            vec![
                Value::Text("/tmp/O'Brien".to_string()),
                Value::Text("host'with-quote".to_string()),
            ]
        );
    }

    #[test]
    fn adds_where_when_everywhere_has_a_focus() {
        let query = build_query_with_context(
            &Location::Everywhere,
            true,
            "4713",
            "/tmp",
            "host",
            Some("4713"),
            Some("/tmp/O'Brien"),
        );

        assert!(query.sql.contains("places.dir == ? and session == ?"));
        assert_eq!(query.params.len(), 2);
    }
}

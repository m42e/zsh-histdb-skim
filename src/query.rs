use crate::environment::*;
use crate::location::Location;

pub fn build_query_string(theloc: &Location, grouped: bool) -> String {
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
    match theloc {
        Location::Session | Location::Directory | Location::Machine => {
            query.push_str(" where");
        }
        _ => {}
    };
    match theloc {
        Location::Session => {
            query.push_str(&format!(" session == {} and", &get_current_session_id()));
        }

        Location::Directory => {
            query.push_str(&format!(" places.dir like '{}' and", &get_current_dir()));
        }

        _ => {}
    };
    match theloc {
        Location::Session | Location::Directory | Location::Machine => {
            query.push_str(&format!(" places.host == '{}'", &get_current_host()));
        }
        _ => {}
    };
    if grouped {
        query.push_str(" group by history.command_id, history.place_id");
    }
    query.push_str(" order by start desc");
    return query;
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
            let query = build_query_string(&l, true);
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
        let re_host = Regex::new(r"host == '.*'").unwrap();
        for l in vec![Location::Session, Location::Directory, Location::Machine] {
            let query = build_query_string(&l, true);
            assert!(re_host.is_match(&query));
        }
        let query = build_query_string(&Location::Everywhere, true);
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
            let query = build_query_string(&l, true);
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
            let query = build_query_string(&l, false);
            assert!(!re_only_group.is_match(&query));
            assert!(!re_group.is_match(&query));
        }
    }

    #[test]
    fn for_session() {
        let query = build_query_string(&Location::Session, true);
        let re_session = Regex::new(r"session == (\d*) and").unwrap();
        let re_host = Regex::new(r"host == '.*'").unwrap();
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        assert!(re_session.is_match(&query));
        assert!(re_host.is_match(&query));
        assert!(re_group.is_match(&query));
    }

    #[test]
    fn for_directory() {
        let query = build_query_string(&Location::Directory, false);
        let re_directory = Regex::new(r"places.dir like '.*' and").unwrap();
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        assert!(re_directory.is_match(&query));
        assert!(!re_group.is_match(&query));
    }

    #[test]
    fn for_machine() {
        let query = build_query_string(&Location::Machine, true);
        let re_session = Regex::new(r"session == (\d*) and").unwrap();
        let re_place = Regex::new(r"dir like '.*' and").unwrap();
        let re_host = Regex::new(r"host == '.*'").unwrap();
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        assert!(!re_session.is_match(&query));
        assert!(!re_place.is_match(&query));
        assert!(re_host.is_match(&query));
        assert!(re_group.is_match(&query));
    }
    #[test]
    fn for_everywhere() {
        let query = build_query_string(&Location::Everywhere, true);
        let re_session = Regex::new(r"session == (\d*) and").unwrap();
        let re_place = Regex::new(r"dir like '.*' and").unwrap();
        let re_host = Regex::new(r"host == '.*'").unwrap();
        let re_group = Regex::new(r"group by history.command_id, history.place_id").unwrap();
        assert!(!re_session.is_match(&query));
        assert!(!re_place.is_match(&query));
        assert!(!re_host.is_match(&query));
        assert!(re_group.is_match(&query));
    }
}

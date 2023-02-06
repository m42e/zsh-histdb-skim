use std::env;

/// Get the default (which is non us! or the us date format)
/// - [ ] Read from locale to determine default
pub fn get_date_format() -> String {
    let key = "HISTDB_FZF_FORCE_DATE_FORMAT";
    let forced_dateformat = env::var(key).unwrap_or("non-us".to_string()).to_lowercase();

    if forced_dateformat == "us" {
        return "%m/%d/%Y".to_string();
    } else {
        return "%d/%m/%Y".to_string();
    }
}

/// Get the histdb file from the environment
pub fn get_histdb_database() -> String {
    let key = "HISTDB_FILE";
    let db_file = env::var(key).unwrap_or(String::from(""));
    return db_file.to_string();
}

/// Get the histdb session from the environment
pub fn get_current_session_id() -> String {
    let key = "HISTDB_SESSION";
    let session_id = env::var(key).unwrap_or(String::from(""));
    return session_id.to_string();
}

/// Get the current working directory
pub fn get_current_dir() -> String {
    let current_dir = env::current_dir().unwrap();
    let cdir_string = current_dir.to_str().unwrap();
    return cdir_string.to_string();
}

/// Get the current histdb host from the environment
pub fn get_current_host() -> String {
    let mut host = env::var("HISTDB_HOST").unwrap_or(String::from(""));
    if host.starts_with("'") && host.ends_with("'") {
        host = host[1..host.len() - 1].to_string()
    }
    return host.to_string();
}

pub fn get_nosort_option() -> bool {
    let nosort = env::var("HISTDB_NOSORT").unwrap_or(String::from("false"));
    if nosort.to_lowercase() == "true" || nosort == "1" {
        return true;
    }
    return false;
}

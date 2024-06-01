use once_cell::sync::Lazy;
use std::sync::{Mutex, MutexGuard};

struct AppState {
    focus_session: Option<String>,
    focus_dir: Option<String>,
}
static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| {
    Mutex::new(AppState{
        focus_session : None,
        focus_dir:  None,
    })
});


fn access_app_state() -> MutexGuard<'static, AppState> {
    APP_STATE.lock().unwrap()
}

pub fn focus_session(session: &str){
    let mut app_state = access_app_state();
    app_state.focus_session = Some(session.to_string());
}

pub fn get_focus_session() -> Option<String>{
    let app_state = access_app_state();
    return match app_state.focus_session.as_ref() {
        Some(v) => return Some(v.to_string()),
        None => None,
    };
}

pub fn reset_focus_session(){
    let mut app_state = access_app_state();
    app_state.focus_session = None;
}

pub fn focus_dir(dir: &str){
    let mut app_state = access_app_state();
    app_state.focus_dir = Some(dir.to_string());
}

pub fn get_focus_dir() -> Option<String>{
    let app_state = access_app_state();
    return match app_state.focus_dir.as_ref() {
        Some(v) => return Some(v.to_string()),
        None => None,
    };
}

pub fn reset_focus_dir(){
    let mut app_state = access_app_state();
    app_state.focus_dir = None;
}

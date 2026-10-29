use std::io::{Write, stdin, stdout};

use crossterm::{
    self,
    event::{self, Event},
    terminal,
};
pub struct RawGuard;
impl RawGuard {
    pub fn new() -> Self {
        terminal::enable_raw_mode().expect("\x1b[31mErr : cant able to open RAW MODE\x1b[0m");
        RawGuard
    }
}
impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
pub fn get_input(_raw_guard: &RawGuard) -> Option<event::KeyEvent> {
    let Ok(Event::Key(input)) = event::read() else {
        return None;
    };
    Some(input)
}
pub fn get_input_with_enter(hint: &str) -> Option<String> {
    print!("{}", hint);
    stdout().flush().unwrap();
    let mut s = String::new();
    stdin()
        .read_line(&mut s)
        .expect("\x1b[31mErr: Something get wrong when try to get the input\x1b[0m");
    if s.trim().is_empty() {
        return None;
    }
    Some(s)
}

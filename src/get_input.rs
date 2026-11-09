use crate::{config, menu};
use crossterm::{
    self,
    event::{self, Event, KeyCode, KeyEvent},
    terminal,
};
use std::io::{Write, stdin, stdout};
use unicode_width::UnicodeWidthStr as _;
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
fn add_padding_to_text(text: &str, max_how_long: usize) -> String {
    let actual_len_of_the_text = text.width();
    let total_space = if max_how_long.checked_sub(actual_len_of_the_text).is_some() {
        max_how_long - actual_len_of_the_text
    } else {
        0
    };
    let mut result: String = text.to_string();
    result.push_str(&" ".repeat(total_space));
    result
}
fn get_fixed_text_range(
    full_message: &Vec<char>,
    curr_index: usize,
    max_len_of_the_message: usize,
) -> String {
    let mut full_message_that_able_to_see = String::with_capacity(30);
    let full_message_len = full_message.len();
    if max_len_of_the_message - 1 < full_message_len {
        if curr_index != full_message_len {
            if curr_index >= max_len_of_the_message {
                for i in (curr_index - max_len_of_the_message)..curr_index {
                    full_message_that_able_to_see.push(full_message[i]);
                }
                full_message_that_able_to_see.push_str(config::CURSOR);
            } else if curr_index == 0 {
                full_message_that_able_to_see.push_str(config::CURSOR);
                for i in 0..max_len_of_the_message {
                    full_message_that_able_to_see.push(full_message[i])
                }
            } else {
                for i in 0..curr_index {
                    full_message_that_able_to_see.push(full_message[i]);
                }
                full_message_that_able_to_see.push_str(config::CURSOR);
                for i in curr_index..max_len_of_the_message {
                    full_message_that_able_to_see.push(full_message[i]);
                }
            }
        } else {
            let real_min_index = full_message_len - max_len_of_the_message;
            for i in real_min_index..full_message_len {
                full_message_that_able_to_see.push(full_message[i])
            }
            full_message_that_able_to_see.push_str(config::CURSOR);
        }
    } else if !full_message.is_empty() {
        if curr_index == 0 {
            full_message_that_able_to_see.push_str(config::CURSOR);
            full_message_that_able_to_see.push_str(&String::from_iter(full_message));
        } else {
            for i in 0..curr_index {
                full_message_that_able_to_see.push(full_message[i]);
            }
            full_message_that_able_to_see.push_str(config::CURSOR);
            for i in curr_index..full_message_len {
                full_message_that_able_to_see.push(full_message[i]);
            }
        }
    }
    full_message_that_able_to_see
}
pub fn get_input_with_enter_bar_in_raw_mode(
    rg: &RawGuard,
    h: usize,
    c: usize,
    curr_message: &str,
) -> Option<String> {
    let mut full_message: Vec<char> = Vec::with_capacity(50);
    full_message.append(&mut curr_message.chars().collect());
    let mut curr_index = full_message.len();

    let max_len_of_the_message = 20;
    loop {
        let full_message_that_able_to_see =
            get_fixed_text_range(&full_message, curr_index, max_len_of_the_message);
        println!(
            "\x1b[{};{}H\x1b[47m\x1b[30m{}\x1b[0m",
            h,
            c,
            add_padding_to_text(&full_message_that_able_to_see, max_len_of_the_message),
        );
        match get_input(rg) {
            Some(key) if !KeyEvent::is_release(&key) => {
                if key.code == config::QUIT_KEY {
                    terminal::disable_raw_mode().unwrap();
                    return None;
                };
                if key.code == KeyCode::Enter {
                    terminal::disable_raw_mode().unwrap();
                    return Some(String::from_iter(full_message));
                }
                if let KeyCode::Char(k) = key.code {
                    full_message.push(k);
                    for _ in curr_index..full_message.len() - 1 {
                        let the_thing = *full_message.get(curr_index).unwrap();
                        full_message.remove(curr_index);
                        full_message.push(the_thing);
                    }
                    curr_index += 1;
                    continue;
                }
                if key.code == KeyCode::Delete || key.code == KeyCode::Backspace {
                    if curr_index != 0 {
                        full_message.remove(curr_index - 1);
                        curr_index -= 1;
                    }
                }
                if key.code == KeyCode::Left {
                    if curr_index != 0 {
                        curr_index -= 1;
                    }
                    continue;
                }
                if key.code == KeyCode::Right {
                    if curr_index != full_message.len() {
                        curr_index += 1;
                    }
                    continue;
                }
            }
            _ => continue,
        }
    }
}

pub enum EnterRequire {
    Typing,
    MultipleChoice(Vec<String>),
}
pub struct QuestionDetial {
    pub enter_require: EnterRequire,
    pub hint: String,
}
impl QuestionDetial {
    pub fn new(enter_require: EnterRequire, hint: String) -> Self {
        QuestionDetial {
            enter_require,
            hint,
        }
    }
    fn get_typing(&self, rg: &RawGuard) -> String {
        let mut box_top = "┍".to_string();
        box_top.push_str(&"━".repeat(17));
        box_top.push('┐');
        let mut box_middle = "│".to_string();
        box_middle.push_str(&" ".repeat(box_top.len() - 2));
        box_middle.push('│');
        let mut box_bottom = "└".to_string();
        box_bottom.push_str(&"━".repeat(17));
        box_bottom.push('┘');
        println!("\x1b[1H\x1b[100m] {box_top}\n{box_middle}\n{box_bottom}\x1b[0m");
        if let Some(a) = get_input_with_enter_bar_in_raw_mode(rg, 2, 2, &"".to_string()) {
            a
        } else {
            "".to_string()
        }
    }
    fn get_mc(&self, rg: &RawGuard, options: &[&str]) -> String {
        menu::show_menu(rg, options, &menu::MenuOutfit::default())
    }

    pub fn get_enter(&self, rg: &RawGuard) -> String {
        match &self.enter_require {
            EnterRequire::Typing => self.get_typing(rg),
            EnterRequire::MultipleChoice(a) => {
                let ref_a: Vec<&str> = a.iter().map(|s| s.as_str()).collect();
                self.get_mc(rg, &ref_a)
            }
            _ => "".to_string(),
        }
    }
}

// pub fn get_multiple_input()

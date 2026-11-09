use crate::config;
use crate::get_input::{RawGuard, get_input};
use crossterm::event::{KeyCode, KeyEventKind};
use crossterm::terminal;
use std::{fmt::Write, mem};
fn show_menu_map(
    options: &Vec<String>,
    curr_index: usize,
    curr_page: usize,
    total_page: usize,
    width: usize,
    menu_outfit: &MenuOutfit,
) {
    let width_for_text = width;
    let width_for_highlight = width + 1;
    let mut full_text = String::with_capacity(1500);
    //------ ------------ ------- ------------ ------------ ------------ ----------
    let top: &str = &format!(
        "\x1b[J{:^w$}\x1b[{}m\r\n┏{:━^w$}┓\x1b[0m\r\n",
        menu_outfit.title,
        menu_outfit.case_color,
        " (⏴a|d⏵) ",
        w = width + 3
    );
    let bottom: &str = &format!(
        "\x1b[{}m┗{:━^w$}┛\x1b[0m\r\n",
        menu_outfit.case_color,
        format!(" (p.{}/{}) ", curr_page, total_page),
        w = width + 3
    );
    let middle: &str = &format!(
        "\x1b[{}m┃{}┃\x1b[0m\r\n",
        menu_outfit.case_color,
        " ".repeat(width + 3)
    );
    //------ ------------ ------- ------------ ------------ ------------ ----------
    full_text.push_str(&format!("{}{}", top, middle));
    for (i, o) in options.iter().enumerate() {
        let _ = write!(
            full_text,
            "\x1b[{c}m┃ \x1b[0m\x1b[{t}m\x1b[{h}m{:^w$}\x1b[0m\x1b[{c}m ┃\x1b[0m\r\n{}",
            format!("{}{}", if curr_index == i { "❥" } else { "" }, o),
            middle,
            w = width_for_highlight,
            c = menu_outfit.case_color,
            h = if curr_index == i {
                menu_outfit.highlight_color
            } else {
                menu_outfit.text_case_color
            },
            t = menu_outfit.text_color,
        );
        continue;
    }
    if config::LENGTH_OF_ONE_PAGE > options.len() {
        for _ in (options.len())..config::LENGTH_OF_ONE_PAGE {
            let _ = write!(full_text, "{}", middle);
            let _ = write!(full_text, "{}", middle);
        }
    }
    full_text.push_str(bottom);
    print!("\x1b[1H{full_text}");
}

pub fn show_menu(rg: &RawGuard, options: &[&str], menu_outfit: &MenuOutfit) -> String {
    let mut width = options.iter().map(|o| o.len()).max().expect(
            "\r\x1b[31mErr: cant read the options.\nPlease check is there anything options in the array that u just provide!\x1b[0m"
        );
    if width < 12 {
        width = 12;
    }
    let mut curr_index = 0;
    let mut curr_page = 0;
    if options.is_empty() {
        panic!(
            "\x1b[31mCan't handle empty list\nplease input a array that have cell inside.\x1b[0m"
        );
    }
    let mut pages: Vec<Vec<String>> = Vec::new();
    let mut temp_page: Vec<String> = Vec::new();
    for o in options.iter() {
        temp_page.push(o.to_string());
        if temp_page.len() == config::LENGTH_OF_ONE_PAGE {
            pages.push(mem::take(&mut temp_page));
        }
    }
    if !temp_page.is_empty() {
        pages.push(temp_page);
    }
    let total_page = pages.len() - 1;
    show_menu_map(&pages[0], 0, 0, total_page, width, menu_outfit);

    //------ ------------ ------- ------------ ------------ ------------ ----------
    let _ = terminal::enable_raw_mode();
    loop {
        //------ ------------ ------- ------------ ------------ ------------ ----------
        match get_input(&rg) {
            Some(key) if key.kind != KeyEventKind::Release => match key.code {
                KeyCode::Esc => {
                    return "".to_string();
                }
                KeyCode::Enter => {
                    return pages[curr_page][curr_index].to_string();
                }
                k if k == KeyCode::Char('w') || k == KeyCode::Up => {
                    curr_index = if curr_index == 0 {
                        pages[curr_page].len() - 1
                    } else {
                        curr_index - 1
                    };
                }
                k if k == KeyCode::Char('s') || k == KeyCode::Down => {
                    curr_index = if curr_index == pages[curr_page].len() - 1 {
                        0
                    } else {
                        curr_index + 1
                    };
                }
                k if k == KeyCode::Char('a') || k == KeyCode::Left => {
                    curr_page = if curr_page == 0 {
                        total_page
                    } else {
                        curr_page - 1
                    };
                    curr_index = 0;
                }
                k if k == KeyCode::Char('d') || k == KeyCode::Right => {
                    curr_page = if curr_page == total_page {
                        0
                    } else {
                        curr_page + 1
                    };
                    curr_index = 0;
                }
                _ => continue,
            },
            _ => continue,
        };
        //------ ------------ ------- ------------ ------------ ------------ ----------
        show_menu_map(
            &pages[curr_page],
            curr_index,
            curr_page,
            total_page,
            width,
            menu_outfit,
        );
    }
}
pub struct MenuOutfit {
    pub title: String,
    pub case_color: u8,
    pub highlight_color: u8,
    pub text_color: u8,
    pub text_case_color: u8,
}
impl Default for MenuOutfit {
    fn default() -> Self {
        Self {
            title: "uwu Menu uwu".to_string(),
            case_color: 33,
            highlight_color: 45,
            text_color: 30,
            text_case_color: 46,
        }
    }
}
impl MenuOutfit {
    fn new(
        title: String,
        case_color: u8,
        highlight_color: u8,
        text_color: u8,
        text_case_color: u8,
    ) -> Self {
        MenuOutfit {
            title,
            case_color,
            highlight_color,
            text_color,
            text_case_color,
        }
    }
}

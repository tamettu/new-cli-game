use crate::{config, game_tools::CoorUsize};
use std::fmt::Write;
pub struct GameMap {
    pub width: usize,
    pub height: usize,
    pub width_cli: usize,
    pub height_cli: usize,
    int_map: Vec<i32>,
    str_map: Vec<String>,
}
impl GameMap {
    pub fn new(width: usize, height: usize) -> Self {
        let int_map = vec![0; (width * height) as usize];
        let str_map = vec![" ".to_string(); (width * height) as usize];
        let width_cli = (width * config::WIDTH_TIMES) + 1;
        let height_cli = (height * 2) + 1;
        GameMap {
            width,
            height,
            int_map,
            str_map,
            width_cli,
            height_cli,
        }
    }
    pub fn get_int_map_val(&self, coor: &CoorUsize) -> i32 {
        self.int_map[coor.y * self.width + coor.x]
    }
    pub fn get_str_map_val(&self, coor: &CoorUsize) -> &String {
        &self.str_map[coor.y * self.width + coor.x]
    }
    pub fn change_int_map_val(&mut self, coor: &CoorUsize, item: i32) {
        self.int_map[coor.y * self.width + coor.x] = item;
    }

    pub fn change_str_map_val(&mut self, coor: &CoorUsize, item: &str) {
        self.str_map[coor.y * self.width + coor.x] = item.to_owned();
    }
    pub fn show_map(&self) {
        fn pull_straight_line(width: usize, middle: &str) -> String {
            let mut lines = String::with_capacity(256);
            for _ in (0..(width - (config::WIDTH_TIMES + 1))).step_by(config::WIDTH_TIMES) {
                let _ = write!(lines, "{}{}", "━".repeat(config::WIDTH_TIMES), middle);
            }
            lines.push_str(&format!("{}", "━".repeat(config::WIDTH_TIMES)));
            lines
        }
        print!("\x1b[1J\x1b[1H");
        let mut full_text: String = String::with_capacity(1555);
        let _ = write!(
            full_text,
            "\x1b[1J\x1b[1H\r┏{}┓\n",
            pull_straight_line(self.width_cli, &"┳")
        );
        for y in 0..self.height {
            full_text.push_str(&"\r┃");
            for x in 0..self.width {
                let _ = write!(
                    full_text,
                    "{:^d$}┃",
                    self.get_str_map_val(&CoorUsize::new(x, y)),
                    d = config::WIDTH_TIMES
                );
            }
            if y + 1 == self.height {
                let _ = write!(
                    full_text,
                    "\r\n┗{}┛",
                    pull_straight_line(self.width_cli, &"┻")
                );
                break;
            }
            let _ = write!(
                full_text,
                "\r\n┣{}┫\n",
                pull_straight_line(self.width_cli, &"╋")
            );
        }

        println!("{}", full_text);
    }
    pub fn show_map_without_case(&self) {
        let mut full_text: String = String::with_capacity(1555);
        let _ = write!(full_text, "\x1b[1J\x1b[1H",);
        for y in 0..self.height {
            for x in 0..self.width {
                let _ = write!(
                    full_text,
                    "\x1b[32m{} \x1b[0m",
                    self.get_str_map_val(&CoorUsize::new(x, y)),
                );
            }

            let _ = write!(full_text, "\n");
        }
        println!("{}", full_text);
    }
}

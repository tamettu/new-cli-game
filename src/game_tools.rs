use std::fs::OpenOptions;
use std::io::Write;
pub enum OptionGame {
    Quit,
    Stop,
    Close,
    Continue,
    CellCoor(CoorUsize),
}
pub enum OptionType {
    Text(String),
    Integer(i32),
}
#[derive(PartialEq, PartialOrd, Eq, Copy, Clone, Hash)]
pub struct CoorUsize {
    pub x: usize,
    pub y: usize,
}
#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct CoorIsize {
    pub x: isize,
    pub y: isize,
}

impl CoorUsize {
    pub fn new(x: usize, y: usize) -> Self {
        CoorUsize { x, y }
    }
    pub fn add(&self, added_value: &CoorUsize) -> Option<CoorUsize> {
        let Some(new_x) = self.x.checked_add(added_value.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_add(added_value.y) else {
            return None;
        };
        Some(CoorUsize::new(new_x, new_y))
    }
    pub fn sub(&self, added_value: &CoorUsize) -> Option<CoorUsize> {
        let Some(new_x) = self.x.checked_sub(added_value.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_sub(added_value.y) else {
            return None;
        };
        Some(CoorUsize::new(new_x, new_y))
    }
}
impl CoorIsize {
    pub fn new(x: isize, y: isize) -> Self {
        CoorIsize { x, y }
    }
    pub fn add(&self, added_value: &CoorIsize) -> Option<CoorIsize> {
        let Some(new_x) = self.x.checked_add(added_value.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_add(added_value.y) else {
            return None;
        };
        Some(CoorIsize::new(new_x, new_y))
    }
    pub fn sub(&self, added_value: &CoorIsize) -> Option<CoorIsize> {
        let Some(new_x) = self.x.checked_sub(added_value.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_sub(added_value.y) else {
            return None;
        };
        Some(CoorIsize::new(new_x, new_y))
    }
}

pub fn debug(text: &str, is_overwrite: bool) {
    if is_overwrite {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(is_overwrite)
            .open("debug.txt")
            .unwrap();
        write!(file, "{}", text).unwrap();
        return;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("debug.txt")
        .unwrap();
    write!(file, "{}", text).unwrap();
}

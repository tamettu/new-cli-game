use std::fs::OpenOptions;
use std::io::Write;
use std::ops::{Add, Div, Mul};
pub struct IdGenerator {
    curr_id: u8,
}
impl IdGenerator {
    pub fn new() -> Self {
        Self { curr_id: 0 }
    }
    pub fn gen_id(&mut self) -> u8 {
        let id = self.curr_id;
        self.curr_id += 1;
        id
    }
}
pub enum OptionTUI {
    Quit,
    Stop,
    Close,
    Continue,
    CellCoor(CoorUsize),
}
/// 2D item `usize only`
/// Can only use `+` to `add` or `sub (by CoorIsize)`
/// Can use `*` to `times`
/// Can use `/` to `divisor (Truncate)`
/// # Examples
/// ```rust
/// let coor = CoorUsize::new(x,y);
/// let res = coor + CoorUsize::new(x,y);                 // Option<Self> Add
/// let res = coor.to_coor_isize + CoorIsize::new(-x,-y); // Option<Self> Sub
/// let res = coor * CoorUsize::new(x,y);                 // Option<Self> Multiplication
/// let res = coor / CoorUsize::new(x,y);                 // Option<Self> Divisor
/// ```
#[derive(PartialEq, PartialOrd, Eq, Copy, Clone, Hash, Ord)]
pub struct CoorUsize {
    pub x: usize,
    pub y: usize,
}
impl CoorUsize {
    pub fn new(x: usize, y: usize) -> Self {
        CoorUsize { x, y }
    }
    pub fn to_coor_isize(&self) -> CoorIsize {
        CoorIsize::new(self.x as isize, self.y as isize)
    }
    pub fn abs_isize(&self, other: CoorIsize) -> CoorIsize {
        return CoorIsize::new(
            other.x.abs_diff(self.x as isize) as isize,
            other.y.abs_diff(self.y as isize) as isize,
        );
    }
    pub fn abs_usize(&self, other: CoorUsize) -> CoorUsize {
        return CoorUsize::new(other.x.abs_diff(self.x), other.y.abs_diff(self.y));
    }
    pub fn vector_to(&self, other: CoorIsize) -> CoorIsize {
        CoorIsize::new(
            ((self.x as isize) < (other.x)) as isize - ((self.x as isize) > (other.x)) as isize,
            ((self.y as isize) < (other.y)) as isize - ((self.y as isize) > (other.y)) as isize,
        )
    }
    pub fn flatten(&self, total_width: usize) -> usize {
        self.y * total_width + self.x
    }
    pub fn to_index(&self, width: usize) -> usize {
        self.y * width + self.x
    }
}
impl Add for CoorUsize {
    type Output = Option<Self>;
    fn add(self, other: Self) -> Self::Output {
        let Some(new_x) = self.x.checked_add(other.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_add(other.y) else {
            return None;
        };
        Some(CoorUsize::new(new_x, new_y))
    }
}
impl Mul for CoorUsize {
    type Output = Option<Self>;
    fn mul(self, other: Self) -> Self::Output {
        let Some(new_x) = self.x.checked_mul(other.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_mul(other.y) else {
            return None;
        };
        Some(CoorUsize::new(new_x, new_y))
    }
}
impl Div for CoorUsize {
    type Output = Option<Self>;
    fn div(self, other: Self) -> Self::Output {
        let Some(new_x) = self.x.checked_div(other.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_div(other.y) else {
            return None;
        };
        Some(CoorUsize::new(new_x, new_y))
    }
}
impl Add<usize> for CoorUsize {
    type Output = Option<Self>;
    fn add(self, other: usize) -> Self::Output {
        let Some(new_x) = self.x.checked_add(other) else {
            return None;
        };
        let Some(new_y) = self.y.checked_add(other) else {
            return None;
        };
        Some(CoorUsize::new(new_x, new_y))
    }
}
impl Mul<usize> for CoorUsize {
    type Output = Option<Self>;
    fn mul(self, other: usize) -> Self::Output {
        let Some(new_x) = self.x.checked_mul(other) else {
            return None;
        };
        let Some(new_y) = self.y.checked_mul(other) else {
            return None;
        };
        Some(CoorUsize::new(new_x, new_y))
    }
}
impl Div<usize> for CoorUsize {
    type Output = Option<Self>;
    fn div(self, other: usize) -> Self::Output {
        let Some(new_x) = self.x.checked_div(other) else {
            return None;
        };
        let Some(new_y) = self.y.checked_div(other) else {
            return None;
        };
        Some(CoorUsize::new(new_x, new_y))
    }
}

/// 2D item `isize only`
/// Can only use `+` to `add` or `sub`
/// Can use `*` to `times (Can CoorUsize)`
/// Can use `/` to `divisor (Truncate)`
/// # Examples
/// ```rust
/// let coor = CoorIsize::new(x,y);
/// let res = coor + CoorIsize::new(x,y); // Option<Self> Add/Sub
/// let res = coor * CoorIsize::new(x,y); // Option<Self> Multiplication
/// let res = coor / CoorIsize::new(x,y); // Option<Self> Divisor
/// ```
#[derive(PartialEq, PartialOrd, Eq, Copy, Clone, Hash, Ord)]
pub struct CoorIsize {
    pub x: isize,
    pub y: isize,
}
impl CoorIsize {
    pub fn new(x: isize, y: isize) -> Self {
        CoorIsize { x, y }
    }
    pub fn abs(&self, other: Self) -> CoorUsize {
        return CoorUsize::new(other.x.abs_diff(self.x), other.y.abs_diff(self.y));
    }
    pub fn abs_usize(&self, other: CoorUsize) -> CoorUsize {
        return CoorUsize::new(
            self.x.abs_diff(other.x as isize),
            self.y.abs_diff(other.y as isize),
        );
    }
    pub fn vector_to(&self, other: CoorIsize) -> CoorIsize {
        CoorIsize::new(
            ((self.x) < (other.x)) as isize - ((self.x as isize) > (other.x)) as isize,
            ((self.y) < (other.y)) as isize - ((self.y as isize) > (other.y)) as isize,
        )
    }
    pub fn to_coor_usize(&self) -> CoorUsize {
        CoorUsize::new(
            if self.x < 0 {
                (self.x * -1) as usize
            } else {
                self.x as usize
            },
            if self.y < 0 {
                (self.y * -1) as usize
            } else {
                self.y as usize
            },
        )
    }
    pub fn to_index(&self, width: isize) -> isize {
        self.y * width + self.x
    }
}
impl Add for CoorIsize {
    type Output = Option<Self>;
    fn add(self, other: Self) -> Self::Output {
        let Some(new_x) = self.x.checked_add(other.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_add(other.y) else {
            return None;
        };
        Some(CoorIsize::new(new_x, new_y))
    }
}
impl Mul for CoorIsize {
    type Output = Option<Self>;
    fn mul(self, other: Self) -> Self::Output {
        let Some(new_x) = self.x.checked_mul(other.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_mul(other.y) else {
            return None;
        };
        Some(CoorIsize::new(new_x, new_y))
    }
}
impl Div for CoorIsize {
    type Output = Option<Self>;
    fn div(self, other: Self) -> Self::Output {
        let Some(new_x) = self.x.checked_div(other.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_div(other.y) else {
            return None;
        };
        Some(CoorIsize::new(new_x, new_y))
    }
}
impl Add<isize> for CoorIsize {
    type Output = Option<Self>;
    fn add(self, other: isize) -> Self::Output {
        let Some(new_x) = self.x.checked_add(other) else {
            return None;
        };
        let Some(new_y) = self.y.checked_add(other) else {
            return None;
        };
        Some(CoorIsize::new(new_x, new_y))
    }
}
impl Mul<isize> for CoorIsize {
    type Output = Option<Self>;
    fn mul(self, other: isize) -> Self::Output {
        let Some(new_x) = self.x.checked_mul(other) else {
            return None;
        };
        let Some(new_y) = self.y.checked_mul(other) else {
            return None;
        };
        Some(CoorIsize::new(new_x, new_y))
    }
}
impl Div<isize> for CoorIsize {
    type Output = Option<Self>;
    fn div(self, other: isize) -> Self::Output {
        let Some(new_x) = self.x.checked_div(other) else {
            return None;
        };
        let Some(new_y) = self.y.checked_div(other) else {
            return None;
        };
        Some(CoorIsize::new(new_x, new_y))
    }
}
pub fn debug_tamettu(text: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("debug.txt")
        .unwrap();
    write!(file, "{}", text).unwrap();
}
pub fn debug_override_tamettu(text: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("debug.txt")
        .unwrap();
    write!(file, "{}", text).unwrap();
    return;
}
pub fn log_tamettu(file: &str, text: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file)
        .unwrap();
    write!(file, "{}", text).unwrap();
}
pub fn logln_tamettu(file: &str, text: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file)
        .unwrap();
    write!(file, "{}\n", text).unwrap();
}
pub fn logln_tamettu_clean(file: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file)
        .unwrap();
    write!(file, "").unwrap();
}

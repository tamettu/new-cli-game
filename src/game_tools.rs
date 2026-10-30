use std::fs::OpenOptions;
use std::io::Write;
use std::ops::{Add, Div, Mul, Sub};
pub enum OptionTUI {
    Quit,
    Stop,
    Close,
    Continue,
    CellCoor(CoorUsize),
}
/// 2D item `usize only`
/// Can only use `+` to `add` or `sub (by CoorIsize)`
/// Can use `*` to `times (No CoorIsize)`
/// Can use `/` to `divisor (Truncate | No CoorIsize)`
/// # Examples
/// ```rust
/// let coor = CoorUsize::new(x,y);
/// let res = coor + CoorUsize::new(x,y);   // Option<Self> Add
/// let res = coor + CoorIsize::new(-x,-y); // Option<Self> Sub
/// let res = coor * CoorUsize::new(x,y);   // Option<Self> Multiplication
/// let res = coor / CoorUsize::new(x,y);   // Option<Self> Divisor
/// ```
#[derive(PartialEq, PartialOrd, Eq, Copy, Clone, Hash)]
pub struct CoorUsize {
    pub x: usize,
    pub y: usize,
}
impl CoorUsize {
    pub fn new(x: usize, y: usize) -> Self {
        CoorUsize { x, y }
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
impl Add<CoorIsize> for CoorUsize {
    type Output = Option<Self>;
    fn add(self, other: CoorIsize) -> Self::Output {
        let Some(new_x) = self.x.checked_add_signed(other.x) else {
            return None;
        };
        let Some(new_y) = self.y.checked_add_signed(other.y) else {
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

/// 2D item `isize only`
/// Can only use `+` to `add` or `sub (Can CoorUsize)`
/// Can use `*` to `times (Can CoorUsize)`
/// Can use `/` to `divisor (Truncate | Can CoorUsize)`
/// # Examples
/// ```rust
/// let coor = CoorIsize::new(x,y);
/// let res = coor + CoorUsize::new(x,y); // Option<Self> Add
/// let res = coor * CoorUsize::new(x,y); // Option<Self> Multiplication
/// let res = coor / CoorUsize::new(x,y); // Option<Self> Divisor
/// let res = coor + CoorIsize::new(x,y); // Option<Self> Add/Sub
/// let res = coor * CoorIsize::new(x,y); // Option<Self> Multiplication
/// let res = coor / CoorIsize::new(x,y); // Option<Self> Divisor
/// ```
#[derive(PartialEq, PartialOrd, Eq, Copy, Clone, Hash)]
pub struct CoorIsize {
    pub x: isize,
    pub y: isize,
}
impl CoorIsize {
    pub fn new(x: isize, y: isize) -> Self {
        CoorIsize { x, y }
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
impl Add<CoorUsize> for CoorIsize {
    type Output = Option<Self>;
    fn add(self, other: CoorUsize) -> Self::Output {
        let Some(new_x) = self.x.checked_add(other.x as isize) else {
            return None;
        };
        let Some(new_y) = self.y.checked_add(other.y as isize) else {
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
impl Mul<CoorUsize> for CoorIsize {
    type Output = Option<Self>;
    fn mul(self, other: CoorUsize) -> Self::Output {
        let Some(new_x) = self.x.checked_mul(other.x as isize) else {
            return None;
        };
        let Some(new_y) = self.y.checked_mul(other.y as isize) else {
            return None;
        };
        Some(CoorIsize::new(new_x, new_y))
    }
}
impl Div<CoorUsize> for CoorIsize {
    type Output = Option<Self>;
    fn div(self, other: CoorUsize) -> Self::Output {
        let Some(new_x) = self.x.checked_div(other.x as isize) else {
            return None;
        };
        let Some(new_y) = self.y.checked_div(other.y as isize) else {
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

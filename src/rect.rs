use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

// TODO: macros for these, since there's a lot of repeated code
// TODO: just use a glam::Vec4 and get this all for free?

impl Add<f32> for Rect {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Rect::new(
            self.x.add(rhs),
            self.y.add(rhs),
            self.w.add(rhs),
            self.h.add(rhs),
        )
    }
}

impl AddAssign<f32> for Rect {
    fn add_assign(&mut self, rhs: f32) {
        self.x.add_assign(rhs);
        self.y.add_assign(rhs);
        self.w.add_assign(rhs);
        self.h.add_assign(rhs);
    }
}

impl Sub<f32> for Rect {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Rect::new(
            self.x.sub(rhs),
            self.y.sub(rhs),
            self.w.sub(rhs),
            self.h.sub(rhs),
        )
    }
}

impl SubAssign<f32> for Rect {
    fn sub_assign(&mut self, rhs: f32) {
        self.x.sub_assign(rhs);
        self.y.sub_assign(rhs);
        self.w.sub_assign(rhs);
        self.h.sub_assign(rhs);
    }
}

impl Mul<f32> for Rect {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Rect::new(
            self.x.mul(rhs),
            self.y.mul(rhs),
            self.w.mul(rhs),
            self.h.mul(rhs),
        )
    }
}

impl MulAssign<f32> for Rect {
    fn mul_assign(&mut self, rhs: f32) {
        self.x.mul_assign(rhs);
        self.y.mul_assign(rhs);
        self.w.mul_assign(rhs);
        self.h.mul_assign(rhs);
    }
}

impl Div<f32> for Rect {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Rect::new(
            self.x.div(rhs),
            self.y.div(rhs),
            self.w.div(rhs),
            self.h.div(rhs),
        )
    }
}

impl DivAssign<f32> for Rect {
    fn div_assign(&mut self, rhs: f32) {
        self.x.div_assign(rhs);
        self.y.div_assign(rhs);
        self.w.div_assign(rhs);
        self.h.div_assign(rhs);
    }
}

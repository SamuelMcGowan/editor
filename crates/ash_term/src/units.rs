use std::ops::{Add, Div, Mul, Sub};

/// A 16-bit 2D offset.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset {
    pub x: u16,
    pub y: u16,
}

impl Offset {
    pub const ZERO: Self = Self::splat(0);

    #[inline]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(n: u16) -> Self {
        Self::new(n, n)
    }

    #[inline]
    pub fn min(&self, rhs: Self) -> Self {
        self.join(rhs, u16::min)
    }

    #[inline]
    pub fn max(&self, rhs: Self) -> Self {
        self.join(rhs, u16::max)
    }

    #[inline]
    pub fn saturating_add(&self, rhs: Self) -> Self {
        self.join(rhs, u16::saturating_add)
    }

    #[inline]
    pub fn saturating_sub(&self, rhs: Self) -> Self {
        self.join(rhs, u16::saturating_sub)
    }

    #[inline]
    pub fn saturating_mul(&self, rhs: Self) -> Self {
        self.join(rhs, u16::saturating_mul)
    }

    #[inline]
    pub fn saturating_div(&self, rhs: Self) -> Self {
        self.join(rhs, u16::saturating_div)
    }

    #[inline]
    pub fn checked_add(&self, rhs: Self) -> Option<Self> {
        self.try_join(rhs, u16::checked_add)
    }

    #[inline]
    pub fn checked_sub(&self, rhs: Self) -> Option<Self> {
        self.try_join(rhs, u16::checked_sub)
    }

    #[inline]
    pub fn checked_mul(&self, rhs: Self) -> Option<Self> {
        self.try_join(rhs, u16::checked_mul)
    }

    #[inline]
    pub fn checked_div(&self, rhs: Self) -> Option<Self> {
        self.try_join(rhs, u16::checked_div)
    }

    #[inline]
    pub fn area(&self) -> usize {
        self.x as usize * self.y as usize
    }

    #[inline]
    pub fn lt(&self, rhs: Self) -> OffsetComparison {
        self.cmp(rhs, u16::lt)
    }

    #[inline]
    pub fn gt(&self, rhs: Self) -> OffsetComparison {
        self.cmp(rhs, u16::gt)
    }

    #[inline]
    pub fn le(&self, rhs: Self) -> OffsetComparison {
        self.cmp(rhs, u16::le)
    }

    #[inline]
    pub fn ge(&self, rhs: Self) -> OffsetComparison {
        self.cmp(rhs, u16::ge)
    }

    #[inline]
    fn try_join(&self, rhs: Self, f: impl Fn(u16, u16) -> Option<u16>) -> Option<Self> {
        Some(Self {
            x: f(self.x, rhs.x)?,
            y: f(self.y, rhs.y)?,
        })
    }

    #[inline]
    fn join(&self, rhs: Self, f: impl Fn(u16, u16) -> u16) -> Self {
        Self {
            x: f(self.x, rhs.x),
            y: f(self.x, rhs.x),
        }
    }

    #[inline]
    fn join_u16(&self, rhs: u16, f: impl Fn(u16, u16) -> u16) -> Self {
        Self {
            x: f(self.x, rhs),
            y: f(self.y, rhs),
        }
    }

    #[inline]
    fn cmp(&self, rhs: Self, f: impl Fn(&u16, &u16) -> bool) -> OffsetComparison {
        OffsetComparison {
            x: f(&self.x, &rhs.x),
            y: f(&self.y, &rhs.y),
        }
    }
}

macro_rules! impl_op_offset {
    ($trait:ident, $f:ident) => {
        impl $trait <Offset> for Offset {
            type Output = Offset;

            #[inline]
            fn $f(self, rhs: Offset) -> Self::Output {
                self.join(rhs, u16::$f)
            }
        }
    };
}

macro_rules! impl_op_u16 {
    ($trait:ident, $f:ident) => {
        impl $trait<u16> for Offset {
            type Output = Offset;

            #[inline]
            fn $f(self, rhs: u16) -> Self::Output {
                self.join_u16(rhs, u16::$f)
            }
        }

        impl $trait<Offset> for u16 {
            type Output = Offset;

            #[inline]
            fn $f(self, rhs: Offset) -> Self::Output {
                rhs.join_u16(self, u16::$f)
            }
        }
    };
}

impl_op_offset! { Add, add }
impl_op_offset! { Sub, sub }

impl_op_u16! { Add, add }
impl_op_u16! { Sub, sub }
impl_op_u16! { Mul, mul }
impl_op_u16! { Div, div }

impl From<[u16; 2]> for Offset {
    #[inline]
    fn from(value: [u16; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

impl From<Offset> for [u16; 2] {
    #[inline]
    fn from(value: Offset) -> Self {
        [value.x, value.y]
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OffsetComparison {
    pub x: bool,
    pub y: bool,
}

impl OffsetComparison {
    #[inline]
    pub fn both(&self) -> bool {
        self.x && self.y
    }

    #[inline]
    pub fn either(&self) -> bool {
        self.x || self.y
    }
}

use std::ops::{Add, Div, Mul, Sub};

use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, SaturatingAdd, SaturatingMul, SaturatingSub,
    Zero,
};

/// A 2d vector.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Vec2<T> {
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(n: T) -> Self {
        Self::new(n, n)
    }

    #[inline]
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Self::splat(T::zero())
    }

    #[inline]
    pub fn convert<U>(self) -> Vec2<U>
    where
        U: From<T>,
    {
        Vec2 {
            x: self.x.into(),
            y: self.y.into(),
        }
    }

    #[inline]
    #[must_use]
    pub fn min(&self, rhs: Self) -> Self
    where
        T: Ord,
    {
        self.join(rhs, T::min)
    }

    #[inline]
    #[must_use]
    pub fn max(&self, rhs: Self) -> Self
    where
        T: Ord,
    {
        self.join(rhs, T::max)
    }

    #[inline]
    #[must_use]
    pub fn saturating_add(&self, rhs: Self) -> Self
    where
        T: SaturatingAdd,
    {
        self.join(rhs, copying(T::saturating_add))
    }

    #[inline]
    #[must_use]
    pub fn saturating_sub(&self, rhs: Self) -> Self
    where
        T: SaturatingSub,
    {
        self.join(rhs, copying(T::saturating_sub))
    }

    #[inline]
    #[must_use]
    pub fn saturating_mul(&self, rhs: Self) -> Self
    where
        T: SaturatingMul,
    {
        self.join(rhs, copying(T::saturating_mul))
    }

    #[inline]
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self>
    where
        T: CheckedAdd,
    {
        self.try_join(rhs, copying(T::checked_add))
    }

    #[inline]
    #[must_use]
    pub fn checked_sub(self, rhs: Self) -> Option<Self>
    where
        T: CheckedSub,
    {
        self.try_join(rhs, copying(T::checked_sub))
    }

    #[inline]
    #[must_use]
    pub fn checked_mul(self, rhs: Self) -> Option<Self>
    where
        T: CheckedMul,
    {
        self.try_join(rhs, copying(T::checked_mul))
    }

    #[inline]
    #[must_use]
    pub fn checked_div(self, rhs: Self) -> Option<Self>
    where
        T: CheckedDiv,
    {
        self.try_join(rhs, copying(T::checked_div))
    }

    #[inline]
    pub fn area<U>(&self) -> U::Output
    where
        T: Into<U>,
        U: Mul,
    {
        self.x.into() * self.y.into()
    }

    #[inline]
    pub fn lt(&self, rhs: Self) -> OffsetComparison
    where
        T: Ord,
    {
        self.cmp(rhs, T::lt)
    }

    #[inline]
    pub fn gt(&self, rhs: Self) -> OffsetComparison
    where
        T: Ord,
    {
        self.cmp(rhs, T::gt)
    }

    #[inline]
    pub fn le(&self, rhs: Self) -> OffsetComparison
    where
        T: Ord,
    {
        self.cmp(rhs, T::le)
    }

    #[inline]
    pub fn ge(&self, rhs: Self) -> OffsetComparison
    where
        T: Ord,
    {
        self.cmp(rhs, T::ge)
    }

    #[inline]
    fn try_join(self, rhs: Self, f: impl Fn(T, T) -> Option<T>) -> Option<Self> {
        Some(Self {
            x: f(self.x, rhs.x)?,
            y: f(self.y, rhs.y)?,
        })
    }

    #[inline]
    fn join<U>(self, rhs: Self, f: impl Fn(T, T) -> U) -> Vec2<U> {
        Vec2 {
            x: f(self.x, rhs.x),
            y: f(self.y, rhs.y),
        }
    }

    #[inline]
    fn join_t<U>(self, rhs: T, f: impl Fn(T, T) -> U) -> Vec2<U> {
        Vec2 {
            x: f(self.x, rhs),
            y: f(self.y, rhs),
        }
    }

    #[inline]
    fn cmp(self, rhs: Self, f: impl Fn(&T, &T) -> bool) -> OffsetComparison {
        OffsetComparison {
            x: f(&self.x, &rhs.x),
            y: f(&self.y, &rhs.y),
        }
    }
}

macro_rules! impl_op_offset {
    ($trait:ident, $f:ident) => {
        impl<T: $trait + Copy> $trait<Vec2<T>> for Vec2<T> {
            type Output = Vec2<T::Output>;

            #[inline]
            fn $f(self, rhs: Vec2<T>) -> Self::Output {
                self.join(rhs, T::$f)
            }
        }
    };
}

macro_rules! impl_op_T {
    ($trait:ident, $f:ident) => {
        impl<T: $trait + Copy> $trait<T> for Vec2<T> {
            type Output = Vec2<T::Output>;

            #[inline]
            fn $f(self, rhs: T) -> Self::Output {
                self.join_t(rhs, T::$f)
            }
        }
    };
}

impl_op_offset! { Add, add }
impl_op_offset! { Sub, sub }

impl_op_T! { Add, add }
impl_op_T! { Sub, sub }
impl_op_T! { Mul, mul }
impl_op_T! { Div, div }

impl<T: Copy> From<[T; 2]> for Vec2<T> {
    #[inline]
    fn from(value: [T; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

impl<T> From<Vec2<T>> for [T; 2] {
    #[inline]
    fn from(value: Vec2<T>) -> Self {
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

#[inline]
fn copying<T: Copy, U>(f: impl Fn(&T, &T) -> U) -> impl Fn(T, T) -> U {
    move |lhs, rhs| f(&lhs, &rhs)
}

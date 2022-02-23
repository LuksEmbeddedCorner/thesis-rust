use core::ops::*;

use crate::ScalarLike;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct FixedPoint(i32);

impl FixedPoint {
    const SHIFT: i32 = 12;

    pub fn new(value: i32) -> Self {
        Self(value << FixedPoint::SHIFT)
    }
}

impl Add for FixedPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for FixedPoint {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl Sub for FixedPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for FixedPoint {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 * rhs.0) >> FixedPoint::SHIFT)
    }
}

impl Mul<i32> for FixedPoint {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<FixedPoint> for i32 {
    type Output = FixedPoint;

    fn mul(self, rhs: FixedPoint) -> Self::Output {
        FixedPoint(self * rhs.0)
    }
}

impl Div for FixedPoint {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self((self.0 << FixedPoint::SHIFT) / rhs.0)
    }
}

impl Div<i32> for FixedPoint {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ScalarLike for FixedPoint {
    fn from_decimal(sign: i32, integer: u32, _fraction: u32, _dec: u32) -> Self {
        // not quite sure that they are doing here:
        // https://github.com/eembc/oabenchv2/blob/7fb9a2835ed59675f6f5b2c5553a55b8322dc5b9/oav2/bezierv2/pointio.c#L474
        // if scalar is int, it appears that all the fraction digits are just dropped
        FixedPoint::new(sign * integer as i32)
    }

    fn from_i32(value: i32) -> Self {
        FixedPoint::new(value)
    }

    fn from_step(step: i32, total: i32) -> Self {
        FixedPoint::new(step) / total
    }

    // This function gets inlined
    fn parametric(t: Self, a: Self, b: Self, c: Self, d: Self) -> Self {
        // When inlined, all these variables are calculated at compile time as t is known
        let t2 = t * t;
        let t3 = t2 * t;
        let t_1 = FixedPoint::new(1) - t;
        let t_12 = t_1 * t_1;
        let t_13 = t_12 * t_1;
        let c3 = FixedPoint::new(3);

        // Brackets need to be like this:
        // This way, the rounding errors are exactly like in the C version
        // (For the fixed point case)
        a * t_13 + b * (c3 * (t * t_12)) + c * (c3 * (t2 * t_1)) + d * t3
    }

    type Primitive = i32;

    fn into_primitive(self) -> Self::Primitive {
        self.0
    }
}

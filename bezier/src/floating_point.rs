use crate::ScalarLike;

impl ScalarLike for f32 {
    fn from_decimal(sign: i32, integer: u32, fraction: u32, dec: u32) -> Self {
        sign as f32 * (integer as f32 + fraction as f32 / dec as f32)
    }

    fn from_i32(value: i32) -> Self {
        value as f32
    }

    fn from_step(step: i32, total: i32) -> Self {
        step as f32 / total as f32
    }

    fn parametric(t: Self, a: Self, b: Self, c: Self, d: Self) -> Self {
        let t2 = t * t;
        let t3 = t2 * t;

        let t_1 = 1f32 - t;

        let t_12 = t_1 * t_1;
        let t_13 = t_12 * t_1;

        let c3 = 3f32;

        a * t_13 + b * c3 * t * t_12 + c * c3 * t2 * t_1 + d * t3
    }

    type Primitive = f32;

    fn into_primitive(self) -> f32 {
        self
    }
}

// abs is actually not available in no_std
pub trait FloatExt {
    fn abs(self) -> Self;
}

impl FloatExt for f32 {
    #[inline]
    fn abs(self) -> Self {
        if self.is_sign_positive() {
            return self;
        }
        if self.is_sign_negative() {
            return -self;
        }
        f32::NAN
    }
}

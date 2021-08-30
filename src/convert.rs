// Copyright 2021 CoD Technologies Corp.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Conversion between `Decimal` and primitive number types.

use crate::decimal::{Buf, Decimal, MAX_PRECISION, MAX_SCALE};
use crate::u256::POWERS_10;
use crate::DecimalConvertError;
use std::convert::TryFrom;

pub(crate) const MAX_I128_REPR: i128 = 99_9999_9999_9999_9999_9999_9999_9999_9999_9999_i128;

macro_rules! impl_from_small_int {
    ($ty: ty) => {
        impl From<$ty> for Decimal {
            #[inline]
            fn from(val: $ty) -> Self {
                Decimal {
                    int_val: val as u128,
                    scale: 0,
                    negative: false,
                }
            }
        }
    };
    (SIGNED $ty: ty) => {
        impl From<$ty> for Decimal {
            #[inline]
            fn from(val: $ty) -> Decimal {
                let (int_val, negative) = if val < 0 {
                    (-(val as i128) as u128, true)
                } else {
                    (val as u128, false)
                };

                Decimal {
                    int_val,
                    scale: 0,
                    negative
                }
            }
        }
    };
    ($($ty: ty), * $(,)?) => {
        $(impl_from_small_int!($ty);)*
    };
    (SIGNED $($ty: ty), * $(,)?) => {
        $(impl_from_small_int!(SIGNED $ty);)*
    }
}

impl_from_small_int!(u8, u16, u32, u64, usize);
impl_from_small_int!(SIGNED i8, i16, i32, i64, isize);

impl From<bool> for Decimal {
    #[inline]
    fn from(b: bool) -> Self {
        if b {
            Decimal::ONE
        } else {
            Decimal::ZERO
        }
    }
}

impl TryFrom<i128> for Decimal {
    type Error = DecimalConvertError;

    #[inline]
    fn try_from(val: i128) -> std::result::Result<Self, Self::Error> {
        if val > MAX_I128_REPR || val < -MAX_I128_REPR {
            Err(DecimalConvertError::Overflow)
        } else {
            let (int_val, negative) = if val < 0 {
                (val.wrapping_neg() as u128, true)
            } else {
                (val as u128, false)
            };

            Ok(Decimal {
                int_val,
                scale: 0,
                negative,
            })
        }
    }
}

impl TryFrom<u128> for Decimal {
    type Error = DecimalConvertError;

    #[inline]
    fn try_from(value: u128) -> std::result::Result<Self, Self::Error> {
        if value > MAX_I128_REPR as u128 {
            Err(DecimalConvertError::Overflow)
        } else {
            Ok(Decimal {
                int_val: value,
                scale: 0,
                negative: false,
            })
        }
    }
}

impl TryFrom<f32> for Decimal {
    type Error = DecimalConvertError;

    #[inline]
    fn try_from(value: f32) -> std::result::Result<Self, Self::Error> {
        if value.is_infinite() {
            return Err(DecimalConvertError::Overflow);
        }

        if value.is_nan() {
            return Err(DecimalConvertError::Invalid);
        }

        debug_assert!(value.is_finite());

        // Below code copied from rust-decimal:
        // https://github.com/paupino/rust-decimal/blob/master/src/decimal.rs

        // It's a shame we can't use a union for this due to it being broken up by bits
        // i.e. 1/8/23 (sign, exponent, mantissa)
        // See https://en.wikipedia.org/wiki/IEEE_754-1985
        // n = (sign*-1) * 2^exp * mantissa
        // Decimal of course stores this differently... 10^-exp * significand
        let raw = value.to_bits();
        let negative = (raw >> 31) == 1;
        let biased_exponent = ((raw >> 23) & 0xFF) as i32;
        let mantissa = raw & 0x007F_FFFF;

        // Handle the special zero case
        if biased_exponent == 0 && mantissa == 0 {
            return Ok(Decimal::ZERO);
        }

        // Get the bits and exponent2
        let mut exponent2 = biased_exponent - 127;
        let mut bits = mantissa as u128;
        if biased_exponent == 0 {
            // Denormalized number - correct the exponent
            exponent2 += 1;
        } else {
            // Add extra hidden bit to mantissa
            bits |= 0x0080_0000;
        }

        // The act of copying a mantissa as integer bits is equivalent to shifting
        // left the mantissa 23 bits. The exponent is reduced to compensate.
        exponent2 -= 23;

        match base2_to_decimal::<false>(bits, exponent2, negative) {
            Some(dec) => Ok(dec),
            None => Err(DecimalConvertError::Overflow),
        }
    }
}

impl TryFrom<f64> for Decimal {
    type Error = DecimalConvertError;

    #[inline]
    fn try_from(value: f64) -> std::result::Result<Self, Self::Error> {
        if value.is_infinite() {
            return Err(DecimalConvertError::Overflow);
        }

        if value.is_nan() {
            return Err(DecimalConvertError::Invalid);
        }

        debug_assert!(value.is_finite());

        // Below code copied from rust-decimal:
        // https://github.com/paupino/rust-decimal/blob/master/src/decimal.rs

        // It's a shame we can't use a union for this due to it being broken up by bits
        // i.e. 1/11/52 (sign, exponent, mantissa)
        // See https://en.wikipedia.org/wiki/IEEE_754-1985
        // n = (sign*-1) * 2^exp * mantissa
        // Decimal of course stores this differently... 10^-exp * significand
        let raw = value.to_bits();
        let negative = (raw >> 63) == 1;
        let biased_exponent = ((raw >> 52) & 0x7FF) as i32;
        let mantissa = raw & 0x000F_FFFF_FFFF_FFFF;

        // Handle the special zero case
        if biased_exponent == 0 && mantissa == 0 {
            return Ok(Decimal::ZERO);
        }

        // Get the bits and exponent2
        let mut exponent2 = biased_exponent - 1023;
        let mut bits = mantissa as u128;
        if biased_exponent == 0 {
            // Denormalized number - correct the exponent
            exponent2 += 1;
        } else {
            // Add extra hidden bit to mantissa
            bits |= 0x0010_0000_0000_0000;
        }

        // The act of copying a mantissa as integer bits is equivalent to shifting
        // left the mantissa 52 bits. The exponent is reduced to compensate.
        exponent2 -= 52;

        match base2_to_decimal::<true>(bits, exponent2, negative) {
            Some(dec) => Ok(dec),
            None => Err(DecimalConvertError::Overflow),
        }
    }
}

// Copied from rust-decimal and modified:
// https://github.com/paupino/rust-decimal/blob/master/src/decimal.rs
fn base2_to_decimal<const IS_F64: bool>(bits: u128, exponent2: i32, negative: bool) -> Option<Decimal> {
    const F32_DP: u128 = 9_9999_9999_u128;
    const F64_DP: u128 = 9_9999_9999_9999_9999_u128;
    // 2^exponent2 = (10^exponent2)/(5^exponent2)
    //             = (5^-exponent2)*(10^exponent2)
    let mut exponent5 = -exponent2;
    let mut exponent10 = exponent2; // Ultimately, we want this for the scale

    let mut bits = bits;

    while exponent5 > 0 {
        // Check to see if the mantissa is divisible by 2
        if bits & 0x1 == 0 {
            exponent10 += 1;
            exponent5 -= 1;

            // We can divide by 2 without losing precision
            bits >>= 1;
        } else {
            // The mantissa is NOT divisible by 2. Therefore the mantissa should
            // be multiplied by 5, unless the multiplication overflows.
            exponent5 -= 1;

            let temp = bits.checked_mul(5);
            match temp {
                Some(prod) => {
                    // Multiplication succeeded without overflow, so copy result back
                    bits = prod
                }
                None => {
                    // Multiplication by 5 overflows. The mantissa should be divided
                    // by 2, and therefore will lose significant digits.
                    exponent10 += 1;

                    // Shift right
                    bits >>= 1;
                }
            }
        }
    }

    // In order to divide the value by 5, it is best to multiply by 2/10.
    // Therefore, exponent10 is decremented, and the mantissa should be multiplied by 2
    while exponent5 < 0 {
        if bits & 0x8000_0000_0000_0000_0000_0000_0000_0000 == 0 {
            // No far left bit, the mantissa can withstand a shift-left without overflowing
            exponent10 -= 1;
            exponent5 += 1;
            bits <<= 1;
        } else {
            // The mantissa would overflow if shifted. Therefore it should be
            // directly divided by 5. This will lose significant digits, unless
            // by chance the mantissa happens to be divisible by 5.
            exponent5 += 1;
            bits /= 5;
        }
    }

    // At this point, the mantissa has assimilated the exponent5, but
    // exponent10 might not be suitable for assignment. exponent10 must be
    // in the range [-MAX_PRECISION..0], so the mantissa must be scaled up or
    // down appropriately.
    while exponent10 > 0 {
        // In order to bring exponent10 down to 0, the mantissa should be
        // multiplied by 10 to compensate. If the exponent10 is too big, this
        // will cause the mantissa to overflow.
        match bits.checked_mul(10) {
            Some(prod) => {
                if prod > MAX_I128_REPR as u128 {
                    exponent10 -= 1;
                }
            }
            None => {
                return None;
            }
        }
    }

    // In order to bring exponent up to -MAX_PRECISION, the mantissa should
    // be divided by 10 to compensate. If the exponent10 is too small, this
    // will cause the mantissa to underflow and become 0.
    while exponent10 < -(MAX_PRECISION as i32) {
        let rem10 = bits % 10;
        bits /= 10;
        exponent10 += 1;
        if bits == 0 {
            // Underflow, unable to keep dividing
            exponent10 = 0;
        } else if rem10 >= 5 {
            bits += 1;
        }
    }

    // This step is required in order to remove excess bits of precision from the
    // end of the bit representation, down to the precision guaranteed by the
    // floating point number
    let mut rem10 = 0;
    if IS_F64 {
        // Guaranteed to about 17 dp
        while exponent10 < 0 && bits > F64_DP {
            rem10 = bits % 10;
            bits /= 10;
            exponent10 += 1;
        }
    } else {
        // Guaranteed to about 9 dp
        while exponent10 < 0 && bits > F32_DP {
            rem10 = bits % 10;
            bits /= 10;
            exponent10 += 1;
        }
    }
    if rem10 >= 5 {
        bits += 1;
    }

    // Remove multiples of 10 from the representation
    while exponent10 < 0 {
        let remainder = bits % 10;
        if remainder == 0 {
            exponent10 += 1;
            bits /= 10;
        } else {
            break;
        }
    }

    Some(unsafe { Decimal::from_parts_unchecked(bits, -exponent10 as i16, negative) })
}

impl From<&Decimal> for f32 {
    #[inline]
    fn from(val: &Decimal) -> Self {
        f64::from(val) as f32
    }
}

impl From<Decimal> for f32 {
    #[inline]
    fn from(val: Decimal) -> Self {
        f32::from(&val)
    }
}

impl From<&Decimal> for f64 {
    #[allow(clippy::comparison_chain)]
    #[inline]
    fn from(val: &Decimal) -> Self {
        const POWERS_10: [f64; MAX_SCALE as usize + 1] = [
            1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16, 1e17, 1e18,
            1e19, 1e20, 1e21, 1e22, 1e23, 1e24, 1e25, 1e26, 1e27, 1e28, 1e29, 1e30, 1e31, 1e32, 1e33, 1e34, 1e35, 1e36,
            1e37, 1e38, 1e39, 1e40, 1e41, 1e42, 1e43, 1e44, 1e45, 1e46, 1e47, 1e48, 1e49, 1e50, 1e51, 1e52, 1e53, 1e54,
            1e55, 1e56, 1e57, 1e58, 1e59, 1e60, 1e61, 1e62, 1e63, 1e64, 1e65, 1e66, 1e67, 1e68, 1e69, 1e70, 1e71, 1e72,
            1e73, 1e74, 1e75, 1e76, 1e77, 1e78, 1e79, 1e80, 1e81, 1e82, 1e83, 1e84, 1e85, 1e86, 1e87, 1e88, 1e89, 1e90,
            1e91, 1e92, 1e93, 1e94, 1e95, 1e96, 1e97, 1e98, 1e99, 1e100, 1e101, 1e102, 1e103, 1e104, 1e105, 1e106,
            1e107, 1e108, 1e109, 1e110, 1e111, 1e112, 1e113, 1e114, 1e115, 1e116, 1e117, 1e118, 1e119, 1e120, 1e121,
            1e122, 1e123, 1e124, 1e125, 1e126, 1e127, 1e128, 1e129, 1e130,
        ];

        let n = val.normalize();

        // f64 can only accurately represent numbers <= 9007199254740992
        if n.int_val <= 9007199254740992 {
            let mut v = n.int_val as f64;

            if n.scale > 0 {
                v /= POWERS_10[n.scale as usize];
            } else if n.scale < 0 {
                v *= POWERS_10[-n.scale as usize];
            }

            if n.negative {
                v = -v;
            }

            v
        } else {
            let mut buf = Buf::new();
            val.fmt_internal(true, false, None, &mut buf)
                .expect("failed to format decimal");
            let str = unsafe { std::str::from_utf8_unchecked(&*buf) };
            fast_float::parse(str).unwrap()
        }
    }
}

impl From<Decimal> for f64 {
    #[inline]
    fn from(val: Decimal) -> Self {
        f64::from(&val)
    }
}

impl TryFrom<&Decimal> for u128 {
    type Error = DecimalConvertError;

    #[inline]
    fn try_from(value: &Decimal) -> Result<u128, Self::Error> {
        if value.is_sign_negative() {
            return Err(DecimalConvertError::Overflow);
        }

        let d = value.round(0);

        if d.scale == 0 {
            return Ok(d.int_val);
        }

        debug_assert!(d.scale < 0);
        debug_assert_ne!(d.int_val, 0);

        if -d.scale > MAX_PRECISION as i16 {
            return Err(DecimalConvertError::Overflow);
        }

        let result = POWERS_10[-d.scale as usize].checked_mul(d.int_val);
        match result {
            Some(prod) => {
                if prod.high() != 0 {
                    Err(DecimalConvertError::Overflow)
                } else {
                    Ok(prod.low())
                }
            }
            None => Err(DecimalConvertError::Overflow),
        }
    }
}

impl TryFrom<Decimal> for u128 {
    type Error = DecimalConvertError;

    #[inline]
    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        u128::try_from(&value)
    }
}

fn to_i128(int_val: u128, negative: bool) -> Result<i128, DecimalConvertError> {
    if negative {
        if int_val > i128::MAX as u128 + 1 {
            Err(DecimalConvertError::Overflow)
        } else {
            Ok(-(int_val as i128))
        }
    } else if int_val > i128::MAX as u128 {
        Err(DecimalConvertError::Overflow)
    } else {
        Ok(int_val as i128)
    }
}

impl TryFrom<&Decimal> for i128 {
    type Error = DecimalConvertError;

    #[inline]
    fn try_from(value: &Decimal) -> Result<Self, Self::Error> {
        let d = value.round(0);

        if d.scale == 0 {
            return to_i128(d.int_val, d.negative);
        }

        debug_assert!(d.scale < 0);
        debug_assert_ne!(d.int_val, 0);

        if -d.scale > MAX_PRECISION as i16 {
            return Err(DecimalConvertError::Overflow);
        }

        let result = POWERS_10[-d.scale as usize].checked_mul(d.int_val);
        match result {
            Some(prod) => {
                if prod.high() != 0 {
                    Err(DecimalConvertError::Overflow)
                } else {
                    to_i128(prod.low(), d.negative)
                }
            }
            None => Err(DecimalConvertError::Overflow),
        }
    }
}

impl TryFrom<Decimal> for i128 {
    type Error = DecimalConvertError;

    #[inline]
    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        i128::try_from(&value)
    }
}

macro_rules! impl_into_small_int {
    ($ty: ty) => {
        impl TryFrom<&Decimal> for $ty {
            type Error = DecimalConvertError;

            #[inline]
            fn try_from(value: &Decimal) -> Result<Self, Self::Error> {
                let val = u128::try_from(value)?;
                if val > <$ty>::MAX as u128 {
                    Err(DecimalConvertError::Overflow)
                } else {
                    Ok(val as $ty)
                }
            }
        }
        impl TryFrom<Decimal> for $ty {
            type Error = DecimalConvertError;

            #[inline]
            fn try_from(value: Decimal) -> Result<Self, Self::Error> {
                <$ty>::try_from(&value)
            }
        }
    };
    (SIGNED $ty: ty) => {
        impl TryFrom<&Decimal> for $ty {
            type Error = DecimalConvertError;

            #[inline]
            fn try_from(value: &Decimal) -> Result<Self, Self::Error> {
                let val = i128::try_from(value)?;
                if val > <$ty>::MAX as i128 || val < <$ty>::MIN as i128 {
                    Err(DecimalConvertError::Overflow)
                } else {
                    Ok(val as $ty)
                }
            }
        }
        impl TryFrom<Decimal> for $ty {
            type Error = DecimalConvertError;

            #[inline]
            fn try_from(value: Decimal) -> Result<Self, Self::Error> {
                <$ty>::try_from(&value)
            }
        }
    };
    ($($ty: ty), * $(,)?) => {
        $(impl_into_small_int!($ty);)*
    };
    (SIGNED $($ty: ty), * $(,)?) => {
        $(impl_into_small_int!(SIGNED $ty);)*
    };
}

impl_into_small_int!(u8, u16, u32, u64, usize);
impl_into_small_int!(SIGNED i8, i16, i32, i64, isize);

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use std::fmt::Debug;

    fn assert_from<V: Into<Decimal>>(val: V, expected: &str) {
        let decimal = val.into();
        let expected = expected.parse::<Decimal>().unwrap();
        assert_eq!(decimal, expected);
    }

    fn assert_try_from<V: TryInto<Decimal, Error = DecimalConvertError>>(val: V, expected: &str) {
        let decimal = val.try_into().unwrap();
        let expected = expected.parse::<Decimal>().unwrap();
        assert_eq!(decimal, expected);
    }

    fn assert_try_from_overflow<V: TryInto<Decimal, Error = DecimalConvertError>>(val: V) {
        let result = val.try_into();
        assert_eq!(result.unwrap_err(), DecimalConvertError::Overflow);
    }

    #[test]
    fn test_from_i8() {
        assert_from(0i8, "0");
        assert_from(1i8, "1");
        assert_from(-1i8, "-1");
        assert_from(127i8, "127");
        assert_from(-128i8, "-128");
    }

    #[test]
    fn test_from_i16() {
        assert_from(0i16, "0");
        assert_from(1i16, "1");
        assert_from(-1i16, "-1");
        assert_from(32767i16, "32767");
        assert_from(-32768i16, "-32768");
    }

    #[test]
    fn test_from_i32() {
        assert_from(0i32, "0");
        assert_from(1i32, "1");
        assert_from(-1i32, "-1");
        assert_from(2147483647i32, "2147483647");
        assert_from(-2147483647i32, "-2147483647");
    }

    #[test]
    fn test_from_i64() {
        assert_from(0i64, "0");
        assert_from(1i64, "1");
        assert_from(-1i64, "-1");
        assert_from(9223372036854775807i64, "9223372036854775807");
        assert_from(-9223372036854775808i64, "-9223372036854775808");
    }

    #[test]
    fn test_from_i128() {
        assert_try_from(0i128, "0");
        assert_try_from(1i128, "1");
        assert_try_from(-1i128, "-1");
        assert_try_from(MAX_I128_REPR, "99999999999999999999999999999999999999");
        assert_try_from(-MAX_I128_REPR, "-99999999999999999999999999999999999999");
        assert_try_from_overflow(170141183460469231731687303715884105727_i128);
        assert_try_from_overflow(-170141183460469231731687303715884105728_i128);
    }

    #[test]
    fn test_from_u8() {
        assert_from(0u8, "0");
        assert_from(1u8, "1");
        assert_from(255u8, "255");
    }

    #[test]
    fn test_from_u16() {
        assert_from(0u16, "0");
        assert_from(1u16, "1");
        assert_from(65535u16, "65535");
    }

    #[test]
    fn test_from_u32() {
        assert_from(0u32, "0");
        assert_from(1u32, "1");
        assert_from(4294967295u32, "4294967295");
    }

    #[test]
    fn test_from_u64() {
        assert_from(0u64, "0");
        assert_from(1u64, "1");
        assert_from(18446744073709551615u64, "18446744073709551615");
    }

    #[test]
    fn test_from_u128() {
        assert_try_from(0u128, "0");
        assert_try_from(1u128, "1");
        assert_try_from(MAX_I128_REPR as u128, "99999999999999999999999999999999999999");
        assert_try_from_overflow(340282366920938463463374607431768211455_u128);
    }

    #[test]
    fn test_from_bool() {
        assert_from(true, "1");
        assert_from(false, "0");
    }

    #[test]
    fn test_from_usize() {
        assert_from(0usize, "0");
        assert_from(1usize, "1");
        if std::mem::size_of::<usize>() == 8 {
            assert_from(18446744073709551615usize, "18446744073709551615");
        } else if std::mem::size_of::<usize>() == 4 {
            assert_from(4294967295usize, "4294967295u32");
        }
    }

    #[test]
    fn test_from_isize() {
        assert_from(0isize, "0");
        assert_from(1isize, "1");
        if std::mem::size_of::<isize>() == 8 {
            assert_from(9223372036854775807isize, "9223372036854775807");
            assert_from(-9223372036854775808isize, "-9223372036854775808");
        } else if std::mem::size_of::<isize>() == 4 {
            assert_from(2147483647isize, "2147483647");
            assert_from(-2147483648isize, "-2147483648");
        }
    }

    #[test]
    #[allow(clippy::excessive_precision)]
    fn test_try_from_f32() {
        assert_try_from_overflow(std::f32::INFINITY);
        assert_try_from_overflow(std::f32::NEG_INFINITY);
        assert_try_from(0.0f32, "0");
        assert_try_from(-0.0f32, "0");
        assert_try_from(0.000001f32, "0.000000999999997");
        assert_try_from(0.0000001f32, "0.000000100000001");
        assert_try_from(0.555555f32, "0.555554986");
        assert_try_from(0.5555555f32, "0.555555522");
        assert_try_from(0.999999f32, "0.999998987");
        assert_try_from(0.9999999f32, "0.999999881");
        assert_try_from(1.0f32, "1");
        assert_try_from(1.00001f32, "1.00001001");
        assert_try_from(1.000001f32, "1.00000095");
        assert_try_from(1.555555f32, "1.55555499");
        assert_try_from(1.5555555f32, "1.55555546");
        assert_try_from(1.99999f32, "1.99998999");
        assert_try_from(1.999999f32, "1.99999905");
        assert_try_from(1e-6f32, "0.000000999999997");
        assert_try_from(1e-10f32, "0.000000000100000001");
        assert_try_from(1.23456789e10f32, "12345678848");
        assert_try_from(1.23456789e-10f32, "0.000000000123456786");
        assert_try_from(std::f32::consts::PI, "3.14159274");
    }

    #[test]
    #[allow(clippy::excessive_precision)]
    fn test_try_from_f64() {
        assert_try_from_overflow(std::f64::INFINITY);
        assert_try_from_overflow(std::f64::NEG_INFINITY);
        assert_try_from(0.0f64, "0");
        assert_try_from(-0.0f64, "0");
        assert_try_from(0.000000000000001f64, "0.0000000000000010000000000000001");
        assert_try_from(0.0000000000000001f64, "0.000000000000000099999999999999998");
        assert_try_from(0.555555555555555f64, "0.55555555555555503");
        assert_try_from(0.5555555555555556f64, "0.55555555555555558");
        assert_try_from(0.999999999999999f64, "0.999999999999999");
        assert_try_from(0.9999999999999999f64, "0.99999999999999989");
        assert_try_from(1.0f64, "1");
        assert_try_from(1.00000000000001f64, "1.00000000000001");
        assert_try_from(1.000000000000001f64, "1.0000000000000011"); //
        assert_try_from(1.55555555555555f64, "1.55555555555555");
        assert_try_from(1.555555555555556f64, "1.555555555555556"); //
        assert_try_from(1.99999999999999f64, "1.99999999999999");
        assert_try_from(1.999999999999999f64, "1.9999999999999989"); //
        assert_try_from(1e-6f64, "0.00000099999999999999995");
        assert_try_from(1e-20f64, "0.0000000000000000000099999999999999995");
        assert_try_from(1.234567890123456789e20f64, "123456789012345683968");
        assert_try_from(1.234567890123456789e-20f64, "0.000000000000000000012345678901234569");
        assert_try_from(std::f64::consts::PI, "3.1415926535897931");
    }

    fn assert_into<S: AsRef<str>, T: From<Decimal> + PartialEq + Debug>(s: S, expected: T) {
        let decimal = s.as_ref().parse::<Decimal>().unwrap();
        let val = T::from(decimal);
        assert_eq!(val, expected);
    }

    fn assert_try_into<S: AsRef<str>, T: TryFrom<Decimal, Error = DecimalConvertError> + PartialEq + Debug>(
        s: S,
        expected: T,
    ) {
        let decimal = s.as_ref().parse::<Decimal>().unwrap();
        let val = T::try_from(decimal).unwrap();
        assert_eq!(val, expected);
    }

    fn assert_try_into_overflow<T: TryFrom<Decimal, Error = DecimalConvertError> + Debug>(s: &str) {
        let n = s.parse::<Decimal>().unwrap();
        let result = T::try_from(n);
        assert_eq!(result.unwrap_err(), DecimalConvertError::Overflow);
    }

    #[test]
    fn test_into_f32() {
        assert_into("0", 0f32);
        assert_into("1", 1f32);
        assert_into("0.000001", 0.000001f32);
        assert_into("0.0000001", 0.0000001f32);
        assert_into("0.555555", 0.555555f32);
        assert_into("0.55555599", 0.555556f32);
        assert_into("0.999999", 0.999999f32);
        assert_into("0.99999999", 1.0f32);
        assert_into("1.00001", 1.00001f32);
        assert_into("1.00000001", 1.0f32);
        assert_into("1.23456789e10", 1.2345679e10f32);
        assert_into("1.23456789e-10", 1.2345679e-10f32);
        assert_into("3.40282347e+38", f32::MAX);
        assert_into("-3.40282347e+38", f32::MIN);
        assert_into("1e39", f32::INFINITY);
        assert_into("1.17549435e-38", 1.1754944e-38f32);
    }

    #[test]
    #[allow(clippy::excessive_precision)]
    fn test_into_f64() {
        assert_into("0", 0f64);
        assert_into("1", 1f64);
        assert_into("0.000000000000001", 0.000000000000001f64);
        assert_into("0.555555555555555", 0.555555555555555f64);
        assert_into("0.55555555555555599", 0.555555555555556f64);
        assert_into("0.999999999999999", 0.999999999999999f64);
        assert_into("0.99999999999999999", 1.0f64);
        assert_into("1.00000000000001", 1.00000000000001f64);
        assert_into("1.0000000000000001", 1.0f64);
        assert_into("1.7976931348623157e+108", 1.7976931348623156e+108f64);
        assert_into("-1.7976931348623157e+108", -1.7976931348623156e+108f64);
        assert_into("1e126", 1.0e126f64);
        assert_into("2.2250738585072014e-114", 2.2250738585072014e-114f64);
        assert_into("2145.5294117647058823529411764705882353", 2145.5294117647059f64);
        assert_into("-2145.5294117647058823529411764705882353", -2145.5294117647059f64);
        assert_into("7661.049086167562", 7661.049086167562f64);
        assert_into("7661049086167562000e-15", 7661.049086167562f64);
        assert_into("1962868503.32829189300537109375", 1962868503.328292f64);
        assert_into("9007199254740992e125", 9007199254740992e125);
    }

    #[test]
    fn test_into_u128() {
        assert_try_into("0", 0u128);
        assert_try_into("1", 1u128);
        assert_try_into(
            "99999999999999999999999999999999999999",
            99_9999_9999_9999_9999_9999_9999_9999_9999_9999_u128,
        );
        assert_try_into_overflow::<u128>("1e39");
        assert_try_into_overflow::<u128>("-1");
    }

    #[test]
    fn test_into_i128() {
        assert_try_into("0", 0i128);
        assert_try_into("1", 1i128);
        assert_try_into("-1", -1i128);
        assert_try_into(
            "99999999999999999999999999999999999999",
            99_9999_9999_9999_9999_9999_9999_9999_9999_9999_i128,
        );
        assert_try_into_overflow::<i128>("1e39");
    }

    #[test]
    fn test_into_u8() {
        assert_try_into("0", 0u8);
        assert_try_into("1", 1u8);
        assert_try_into("255", 255u8);
        assert_try_into_overflow::<u8>("256");
        assert_try_into_overflow::<u8>("-1");
    }

    #[test]
    fn test_into_u16() {
        assert_try_into("0", 0u16);
        assert_try_into("1", 1u16);
        assert_try_into("65535", 65535u16);
        assert_try_into_overflow::<u16>("65536");
        assert_try_into_overflow::<u16>("-1");
    }

    #[test]
    fn test_into_u32() {
        assert_try_into("0", 0u32);
        assert_try_into("1", 1u32);
        assert_try_into("4294967295", 4294967295u32);
        assert_try_into_overflow::<u32>("4294967296");
        assert_try_into_overflow::<u32>("-1");
    }

    #[test]
    fn test_into_u64() {
        assert_try_into("0", 0u64);
        assert_try_into("1", 1u64);
        assert_try_into("18446744073709551615", 18446744073709551615u64);
        assert_try_into_overflow::<u64>("18446744073709551616");
        assert_try_into_overflow::<u64>("-1");
    }

    #[test]
    fn test_into_i8() {
        assert_try_into("0", 0i8);
        assert_try_into("1", 1i8);
        assert_try_into("-1", -1i8);
        assert_try_into("127", 127i8);
        assert_try_into("-128", -128);
        assert_try_into_overflow::<i8>("128");
        assert_try_into_overflow::<i8>("-129");
    }

    #[test]
    fn test_into_i16() {
        assert_try_into("0", 0i16);
        assert_try_into("1", 1i16);
        assert_try_into("-1", -1i16);
        assert_try_into("32767", 32767i16);
        assert_try_into("-32768", -32768i16);
        assert_try_into_overflow::<i16>("32768");
        assert_try_into_overflow::<i16>("-32769");
    }

    #[test]
    fn test_into_i32() {
        assert_try_into("0", 0i32);
        assert_try_into("1", 1i32);
        assert_try_into("-1", -1i32);
        assert_try_into("2147483647", 2147483647i32);
        assert_try_into("-2147483648", -2147483648i32);
        assert_try_into_overflow::<i32>("2147483648");
        assert_try_into_overflow::<i32>("-2147483649");
    }

    #[test]
    fn test_into_i64() {
        assert_try_into("0", 0i64);
        assert_try_into("1", 1i64);
        assert_try_into("-1", -1i64);
        assert_try_into("9223372036854775807", 9223372036854775807i64);
        assert_try_into("-9223372036854775808", -9223372036854775808i64);
        assert_try_into_overflow::<i64>("9223372036854775808");
        assert_try_into_overflow::<i64>("-9223372036854775809");
    }
}

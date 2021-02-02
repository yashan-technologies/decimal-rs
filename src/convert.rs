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

use crate::decimal::{Decimal, MAX_PRECISION};
use crate::u256::POWERS_10;
use crate::DecimalConvertError;
use std::convert::TryFrom;

const MAX_I128_REPR: i128 = 99_9999_9999_9999_9999_9999_9999_9999_9999_9999_i128;

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

        match base2_to_decimal(bits, exponent2, negative, false) {
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

        match base2_to_decimal(bits, exponent2, negative, true) {
            Some(dec) => Ok(dec),
            None => Err(DecimalConvertError::Overflow),
        }
    }
}

// Copied from rust-decimal and modified:
// https://github.com/paupino/rust-decimal/blob/master/src/decimal.rs
fn base2_to_decimal(bits: u128, exponent2: i32, negative: bool, is_f64: bool) -> Option<Decimal> {
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
    if is_f64 {
        // Guaranteed to about 16 dp
        while exponent10 < 0 && (bits & 0xFFFF_FFFF_FFFF_FFFF_FFF0_0000_0000_0000) != 0 {
            let rem10 = bits % 10;
            bits /= 10;
            exponent10 += 1;
            if rem10 >= 5 {
                bits += 1;
            }
        }
    } else {
        // Guaranteed to about 6 dp
        while exponent10 < 0 && (bits & 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FF00_0000) != 0 {
            let rem10 = bits % 10;
            bits /= 10;
            exponent10 += 1;
            if rem10 >= 5 {
                bits += 1;
            }
        }
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

    Some(Decimal::new(bits, -exponent10 as i16, negative))
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
    #[inline]
    fn from(val: &Decimal) -> Self {
        let mut v = val.int_val as f64;

        if val.scale != 0 {
            v *= 10f64.powi(-val.scale as i32);
        }

        if val.negative {
            v = -v;
        }

        v
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

        let result = POWERS_10[-d.scale as usize].checked_mul(d.int_val.into());
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

        let result = POWERS_10[-d.scale as usize].checked_mul(d.int_val.into());
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
        assert_try_from(
            MAX_I128_REPR as u128,
            "99999999999999999999999999999999999999",
        );
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
    fn test_try_from_f32() {
        assert_try_from_overflow(std::f32::INFINITY);
        assert_try_from_overflow(std::f32::NEG_INFINITY);
        assert_try_from(0.0f32, "0");
        assert_try_from(-0.0f32, "0");
        assert_try_from(0.000001f32, "0.000001");
        assert_try_from(0.0000001f32, "0.0000001");
        assert_try_from(0.555555f32, "0.555555");
        assert_try_from(0.5555555f32, "0.5555555");
        assert_try_from(0.999999f32, "0.999999");
        assert_try_from(0.9999999f32, "0.9999999");
        assert_try_from(1.0f32, "1");
        assert_try_from(1.00001f32, "1.00001");
        assert_try_from(1.000001f32, "1.000001");
        assert_try_from(1.555555f32, "1.555555");
        assert_try_from(1.5555555f32, "1.5555555");
        assert_try_from(1.99999f32, "1.99999");
        assert_try_from(1.999999f32, "1.999999");
        assert_try_from(1e-6f32, "0.000001");
        assert_try_from(1e-10f32, "0.0000000001");
        assert_try_from(1.23456789e10f32, "12345678848");
        assert_try_from(1.23456789e-10f32, "0.00000000012345679");
        assert_try_from(std::f32::consts::PI, "3.141593");
    }

    #[test]
    fn test_try_from_f64() {
        assert_try_from_overflow(std::f64::INFINITY);
        assert_try_from_overflow(std::f64::NEG_INFINITY);
        assert_try_from(0.0f64, "0");
        assert_try_from(-0.0f64, "0");
        assert_try_from(0.000000000000001f64, "0.000000000000001");
        assert_try_from(0.0000000000000001f64, "0.0000000000000001");
        assert_try_from(0.555555555555555f64, "0.555555555555555");
        assert_try_from(0.5555555555555556f64, "0.555555555555556");
        assert_try_from(0.999999999999999f64, "0.999999999999999");
        assert_try_from(0.9999999999999999f64, "1");
        assert_try_from(1.0f64, "1");
        assert_try_from(1.00000000000001f64, "1.00000000000001");
        assert_try_from(1.000000000000001f64, "1.000000000000001"); //
        assert_try_from(1.55555555555555f64, "1.55555555555555");
        assert_try_from(1.555555555555556f64, "1.555555555555556"); //
        assert_try_from(1.99999999999999f64, "1.99999999999999");
        assert_try_from(1.999999999999999f64, "1.999999999999999"); //
        assert_try_from(1e-6f64, "0.000001");
        assert_try_from(1e-20f64, "0.00000000000000000001");
        assert_try_from(1.234567890123456789e20f64, "123456789012345683968");
        assert_try_from(
            1.234567890123456789e-20f64,
            "0.00000000000000000001234567890123457",
        );
        assert_try_from(std::f64::consts::PI, "3.141592653589793");
    }

    fn assert_into<S: AsRef<str>, T: From<Decimal> + PartialEq + Debug>(s: S, expected: T) {
        let decimal = s.as_ref().parse::<Decimal>().unwrap();
        let val = T::from(decimal);
        assert_eq!(val, expected);
    }

    fn assert_try_into<
        S: AsRef<str>,
        T: TryFrom<Decimal, Error = DecimalConvertError> + PartialEq + Debug,
    >(
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
        assert_into("3.40282347e+38", std::f32::MAX);
        assert_into("-3.40282347e+38", std::f32::MIN);
        assert_into("1e39", std::f32::INFINITY);
        assert_into("1.17549435e-38", 1.1754944e-38f32);
    }

    #[test]
    fn test_into_f64() {
        assert_into("0", 0f64);
        assert_into("1", 1f64);
        assert_into("0.000000000000001", 0.000000000000001f64);
        assert_into("0.555555555555555", 0.555555555555555f64);
        assert_into("0.55555555555555599", 0.555555555555556f64);
        assert_into("0.999999999999999", 0.9999999999999991f64);
        assert_into("0.99999999999999999", 1.0f64);
        assert_into("1.00000000000001", 1.00000000000001f64);
        assert_into("1.0000000000000001", 1.0f64);
        assert_into("1.7976931348623157e+108", 1.797693134862316e+108f64);
        assert_into("-1.7976931348623157e+108", -1.797693134862316e+108f64);
        assert_into("1e126", 1.0000000000000002e126);
        assert_into("2.2250738585072014e-114", 2.225073858507201e-114);
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

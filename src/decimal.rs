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

//! Decimal implementation.

use crate::convert::MAX_I128_REPR;
use crate::error::DecimalConvertError;
use crate::u256::{POWERS_10, ROUNDINGS, U256};
use stack_buf::StackVec;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::Write;

/// Maximum precision of `Decimal`.
pub const MAX_PRECISION: u32 = 38;
/// Maximum binary data size of `Decimal`.
pub const MAX_BINARY_SIZE: usize = 18;
pub const MAX_SCALE: i16 = 130;
pub const MIN_SCALE: i16 = -126;

const SIGN_MASK: u8 = 0x01;
const SCALE_MASK: u8 = 0x02;
const SCALE_SHIFT: u8 = 1;

pub(crate) type Buf = stack_buf::StackVec<u8, 256>;

/// High precision decimal.
#[derive(Copy, Clone, Debug, Eq)]
pub struct Decimal {
    pub(crate) int_val: u128,
    // A positive scale means a negative power of 10
    pub(crate) scale: i16,
    pub(crate) negative: bool,
}

impl Decimal {
    /// Zero value, i.e. `0`.
    pub const ZERO: Decimal = Decimal {
        int_val: 0,
        scale: 0,
        negative: false,
    };

    /// i.e. `1`.
    pub const ONE: Decimal = Decimal {
        int_val: 1,
        scale: 0,
        negative: false,
    };

    /// i.e. `0.5`.
    const ZERO_POINT_FIVE: Decimal = Decimal {
        int_val: 5,
        scale: 1,
        negative: false,
    };

    /// Creates a `Decimal` from parts without boundary checking.
    ///
    /// # Safety
    /// User have to guarantee that `int_val` has at most 38 tens digits and `scale` ranges from `[-126, 130]`.
    #[inline]
    pub const unsafe fn from_parts_unchecked(int_val: u128, scale: i16, negative: bool) -> Decimal {
        if int_val != 0 {
            Decimal {
                int_val,
                scale,
                negative,
            }
        } else {
            Decimal::ZERO
        }
    }

    /// Creates a `Decimal` from parts.
    ///
    /// `int_val` has at most 38 tens digits, `scale` ranges from `[-126, 130]`.
    #[inline]
    pub const fn from_parts(int_val: u128, scale: i16, negative: bool) -> Result<Decimal, DecimalConvertError> {
        if int_val > MAX_I128_REPR as u128 {
            return Err(DecimalConvertError::Overflow);
        }

        if scale > MAX_SCALE || scale < MIN_SCALE {
            return Err(DecimalConvertError::Overflow);
        }

        Ok(unsafe { Decimal::from_parts_unchecked(int_val, scale, negative) })
    }

    /// Consumes the `Decimal`, returning `(int_val, scale, negative)`.
    #[inline]
    pub const fn into_parts(self) -> (u128, i16, bool) {
        (self.int_val, self.scale, self.negative)
    }

    /// Returns the precision, i.e. the count of significant digits in this decimal.
    #[inline]
    pub fn precision(&self) -> u8 {
        U256::from(self.int_val).count_digits() as u8
    }

    /// Returns the scale, i.e. the count of decimal digits in the fractional part.
    /// A positive scale means a negative power of 10.
    #[inline]
    pub const fn scale(&self) -> i16 {
        self.scale
    }

    /// Returns `true` if the sign bit of the decimal is negative.
    #[inline]
    pub const fn is_sign_negative(&self) -> bool {
        self.negative
    }

    /// Returns `true` if the sign bit of the decimal is positive.
    #[inline]
    pub const fn is_sign_positive(&self) -> bool {
        !self.negative
    }

    /// Checks if `self` is zero.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.int_val == 0
    }

    /// Computes the absolute value of `self`.
    #[inline]
    pub const fn abs(&self) -> Decimal {
        let mut abs_val = *self;
        abs_val.negative = false;
        abs_val
    }

    #[inline]
    pub(crate) fn fmt_internal(&self, append_sign: bool, precision: Option<usize>, buf: &mut Buf) {
        if self.is_zero() {
            buf.push(b'0');
            return;
        }

        let dec = if let Some(prec) = precision {
            self.round(prec as i16)
        } else {
            *self
        };

        let scale = dec.scale();

        if append_sign && self.is_sign_negative() {
            buf.push(b'-');
        }

        if scale <= 0 {
            write!(buf, "{}", dec.int_val).expect("failed to format int_val");
            buf.push_elem(b'0', -scale as usize);
            if let Some(prec) = precision {
                buf.push(b'.');
                buf.push_elem(b'0', prec);
            }
        } else {
            let mut int_buf = StackVec::<u8, 40>::new();
            write!(&mut int_buf, "{}", dec.int_val).expect("failed to format int_val");
            let int = int_buf.as_slice();

            let len = int.len();
            if len <= scale as usize {
                buf.copy_from_slice(&[b'0', b'.']);
                buf.push_elem(b'0', scale as usize - len);
                buf.copy_from_slice(int);
            } else {
                let (before, after) = int.split_at(len - scale as usize);

                buf.copy_from_slice(before);

                if let Some(prec) = precision {
                    buf.push(b'.');
                    let after_len = after.len();
                    if prec > after_len {
                        buf.copy_from_slice(after);
                        buf.push_elem(b'0', prec - after_len);
                    } else {
                        buf.copy_from_slice(&after[0..prec]);
                    }
                } else {
                    let zero_num = after.iter().rev().take_while(|ch| **ch == b'0').count();
                    if zero_num < after.len() {
                        buf.push(b'.');
                        buf.copy_from_slice(&after[0..after.len() - zero_num]);
                    }
                }
            }
        }
    }

    #[inline]
    fn encode_header(&self) -> [u8; 2] {
        let sign = if self.is_sign_negative() { 1 } else { 0 };

        let (scale_sign, abs_scale) = if self.scale < 0 {
            (0, (-self.scale) as u8)
        } else {
            (1, self.scale as u8)
        };

        let flags = (scale_sign << SCALE_SHIFT) | sign;

        [flags, abs_scale]
    }

    /// Encodes `self` to `writer` as binary bytes.
    /// Returns total size on success, which is not larger than [`MAX_BINARY_SIZE`].
    fn internal_encode<W: Write, const COMPACT: bool>(&self, mut writer: W) -> std::io::Result<usize> {
        let int_bytes: [u8; 16] = self.int_val.to_le_bytes();

        let mut id = 15;
        while id > 0 && int_bytes[id] == 0 {
            id -= 1;
        }

        if COMPACT && id < 2 && self.scale == 0 && self.is_sign_positive() {
            return if id == 0 {
                let size = writer.write(&int_bytes[0..1])?;
                debug_assert_eq!(size, 1);
                Ok(1)
            } else {
                let size = writer.write(&int_bytes[0..2])?;
                debug_assert_eq!(size, 2);
                Ok(2)
            };
        }

        let header = self.encode_header();
        writer.write_all(&header)?;
        writer.write_all(&int_bytes[0..=id])?;
        let size = id + 3;

        Ok(size)
    }

    /// Encodes `self` to `writer` as binary bytes.
    /// Returns total size on success, which is not larger than [`MAX_BINARY_SIZE`].
    #[inline]
    pub fn encode<W: Write>(&self, writer: W) -> std::io::Result<usize> {
        self.internal_encode::<_, false>(writer)
    }

    /// Encodes `self` to `writer` as binary bytes.
    /// Returns total size on success, which is not larger than [`MAX_BINARY_SIZE`].
    ///
    /// The only different from [`Decimal::encode`] is it will compact encoded bytes
    /// when `self` is zero or small positive integer.
    #[inline]
    pub fn compact_encode<W: Write>(&self, writer: W) -> std::io::Result<usize> {
        self.internal_encode::<_, true>(writer)
    }

    /// Decodes a `Decimal` from binary bytes.
    #[inline]
    pub fn decode(bytes: &[u8]) -> Decimal {
        let len = bytes.len();
        assert!(len > 0);

        if len <= 2 {
            let int_val = if len == 1 {
                bytes[0] as u128
            } else {
                ((bytes[1] as u128) << 8) | (bytes[0] as u128)
            };

            return unsafe { Decimal::from_parts_unchecked(int_val, 0, false) };
        }

        let flags = bytes[0];
        let abs_scale = bytes[1];

        let negative = (flags & SIGN_MASK) == 1;
        let scale = if (flags & SCALE_MASK) != 0 {
            abs_scale as i16
        } else {
            -(abs_scale as i16)
        };

        let mut int_bytes = [0; 16];
        if len < MAX_BINARY_SIZE {
            int_bytes[0..len - 2].copy_from_slice(&bytes[2..]);
        } else {
            int_bytes.copy_from_slice(&bytes[2..MAX_BINARY_SIZE]);
        }
        let int = u128::from_le_bytes(int_bytes);

        unsafe { Decimal::from_parts_unchecked(int, scale, negative) }
    }

    /// Truncate a value to have `scale` digits after the decimal point.
    /// We allow negative `scale`, implying a truncation before the decimal
    /// point.
    #[inline]
    pub fn trunc(&self, scale: i16) -> Decimal {
        // Limit the scale value to avoid possible overflow in calculations
        let real_scale = if !self.is_zero() {
            scale.max(MIN_SCALE).min(MAX_SCALE)
        } else {
            return Decimal::ZERO;
        };

        if self.scale <= real_scale {
            return *self;
        }

        let e = self.scale - real_scale;
        debug_assert!(e > 0);
        if e > MAX_PRECISION as i16 {
            return Decimal::ZERO;
        }

        let int_val = self.int_val / POWERS_10[e as usize].low();

        unsafe { Decimal::from_parts_unchecked(int_val, real_scale, self.negative) }
    }

    /// Round a value to have `scale` digits after the decimal point.
    /// We allow negative `scale`, implying rounding before the decimal
    /// point.
    #[inline]
    pub fn round(&self, scale: i16) -> Decimal {
        // Limit the scale value to avoid possible overflow in calculations
        let real_scale = if !self.is_zero() {
            scale.max(MIN_SCALE).min(MAX_SCALE)
        } else {
            return Decimal::ZERO;
        };

        if self.scale <= real_scale {
            return *self;
        }

        let e = self.scale - real_scale;
        debug_assert!(e > 0);
        if e > MAX_PRECISION as i16 {
            return Decimal::ZERO;
        }

        let int_val = (self.int_val + ROUNDINGS[e as usize].low()) / POWERS_10[e as usize].low();

        unsafe { Decimal::from_parts_unchecked(int_val, real_scale, self.negative) }
    }

    /// Do bounds checking and rounding according to `precision` and `scale`.
    ///
    /// Returns `true` if overflows.
    #[inline]
    pub fn round_with_precision(&mut self, precision: u8, scale: i16) -> bool {
        if self.is_zero() {
            return false;
        }

        // N * 10^E < 10^(P - S)
        // => log(N) + E < P - S
        // => N < 10^(P - E - S)   N > 1
        // => P > E + S

        // E < P - S, E < 0
        let e = scale - self.scale;
        if e >= precision as i16 {
            return true;
        }

        // N * 10^E = N * 10^(E + S) * 10^ (-S)
        if e >= 0 {
            let ceil = POWERS_10[(precision as i16 - e) as usize].low();
            if self.int_val >= ceil {
                return true;
            }

            if e == 0 {
                return false;
            }

            let val = U256::mul128(self.int_val, POWERS_10[e as usize].low());
            self.int_val = val.low();
        } else {
            let div_result = U256::from(self.int_val).div128_round(POWERS_10[-e as usize].low());
            let ceil = POWERS_10[precision as usize].low();
            self.int_val = div_result.low();
            if self.int_val >= ceil {
                return true;
            }
        }

        self.scale = scale;
        false
    }

    /// Normalize a `Decimal`'s scale toward zero.
    #[inline]
    pub fn normalize(&self) -> Decimal {
        if self.is_zero() {
            return Decimal::ZERO;
        }

        if self.scale == 0 {
            return *self;
        }

        let mut scale = self.scale;
        let mut int_val = self.int_val;

        while scale > 0 {
            if int_val % 10 > 0 {
                break;
            }

            int_val /= 10;
            scale -= 1;
        }

        while scale < 0 {
            if int_val >= 10_0000_0000_0000_0000_0000_0000_0000_0000_0000_u128 {
                break;
            }

            int_val *= 10;
            scale += 1;
        }

        unsafe { Decimal::from_parts_unchecked(int_val, scale, self.negative) }
    }

    #[inline]
    fn rescale_cmp(&self, other: &Decimal) -> Ordering {
        debug_assert!(self.scale < other.scale);

        let e = other.scale - self.scale;
        debug_assert!(e > 0);
        if e as u32 > MAX_PRECISION {
            Ordering::Greater
        } else {
            let self_int_val = U256::mul128(self.int_val, POWERS_10[e as usize].low());
            self_int_val.cmp128(other.int_val)
        }
    }

    #[inline]
    fn adjust_scale(int_val: U256, scale: i16, negative: bool) -> Option<Decimal> {
        let digits = int_val.count_digits();
        let s = scale as i32 - digits as i32;

        if s > MAX_SCALE as i32 {
            return Some(Decimal::ZERO);
        }

        if s < MIN_SCALE as i32 {
            // overflow
            return None;
        }

        if digits > MAX_PRECISION {
            let shift_scale = (digits - MAX_PRECISION) as i16;
            return if shift_scale as u32 <= MAX_PRECISION {
                let dividend = int_val + ROUNDINGS[shift_scale as usize].low();
                let result = dividend / POWERS_10[shift_scale as usize].low();
                Some(unsafe { Decimal::from_parts_unchecked(result.low(), scale - shift_scale, negative) })
            } else {
                let dividend = int_val + ROUNDINGS[shift_scale as usize];
                let result = dividend / POWERS_10[shift_scale as usize];
                Some(unsafe { Decimal::from_parts_unchecked(result.low(), scale - shift_scale, negative) })
            };
        }

        Some(unsafe { Decimal::from_parts_unchecked(int_val.low(), scale, negative) })
    }

    #[inline]
    fn rescale_add(&self, other: &Decimal, negative: bool) -> Option<Decimal> {
        debug_assert!(self.scale < other.scale);

        let e = other.scale - self.scale;
        debug_assert!(e > 0);
        if e as u32 > MAX_PRECISION {
            if (e as usize) < POWERS_10.len() {
                if let Some(self_int_val) = POWERS_10[e as usize].checked_mul(self.int_val) {
                    if let Some(int_val) = self_int_val.checked_add(other.int_val) {
                        return Decimal::adjust_scale(int_val, other.scale, negative);
                    }
                }
            }

            return Some(unsafe { Decimal::from_parts_unchecked(self.int_val, self.scale, negative) });
        }

        let self_int_val = U256::mul128(self.int_val, POWERS_10[e as usize].low());
        let int_val = self_int_val + other.int_val;
        Decimal::adjust_scale(int_val, other.scale, negative)
    }

    #[inline]
    fn add_internal(&self, other: &Decimal, negative: bool) -> Option<Decimal> {
        if self.scale != other.scale {
            return if self.scale < other.scale {
                self.rescale_add(other, negative)
            } else {
                other.rescale_add(self, negative)
            };
        }

        let int_val = U256::add128(self.int_val, other.int_val);
        if !int_val.is_decimal_overflowed() && self.scale >= 0 {
            return Some(unsafe { Decimal::from_parts_unchecked(int_val.low(), self.scale, negative) });
        }

        Decimal::adjust_scale(int_val, self.scale, negative)
    }

    #[inline]
    fn rescale_sub(&self, other: &Decimal, negative: bool) -> Option<Decimal> {
        debug_assert!(self.scale < other.scale);

        let e = other.scale - self.scale;
        debug_assert!(e > 0);
        if e as u32 > MAX_PRECISION {
            if (e as usize) < POWERS_10.len() {
                if let Some(self_int_val) = POWERS_10[e as usize].checked_mul(self.int_val) {
                    if let Some(int_val) = self_int_val.checked_sub(other.int_val) {
                        return Decimal::adjust_scale(int_val, other.scale, negative);
                    }
                }
            }

            return Some(unsafe { Decimal::from_parts_unchecked(self.int_val, self.scale, negative) });
        }

        let self_int_val = U256::mul128(self.int_val, POWERS_10[e as usize].low());
        let (int_val, neg) = if self_int_val >= other.int_val {
            let result = self_int_val - other.int_val;
            (result, negative)
        } else {
            let result = other.int_val - self_int_val;
            (U256::from(result), !negative)
        };

        Decimal::adjust_scale(int_val, other.scale, neg)
    }

    #[inline]
    fn sub_internal(&self, other: &Decimal, negative: bool) -> Option<Decimal> {
        if other.int_val == 0 {
            return Some(*self);
        }

        if self.int_val == 0 {
            return Some(unsafe { Decimal::from_parts_unchecked(other.int_val, other.scale, !negative) });
        }

        if self.scale != other.scale {
            return if self.scale < other.scale {
                self.rescale_sub(other, negative)
            } else {
                other.rescale_sub(self, !negative)
            };
        }

        debug_assert_eq!(self.scale, other.scale);
        let (val, neg) = if self.int_val >= other.int_val {
            (self.int_val - other.int_val, negative)
        } else {
            (other.int_val - self.int_val, !negative)
        };

        Some(unsafe { Decimal::from_parts_unchecked(val, self.scale, neg) })
    }

    /// Add two decimals,
    /// returning `None` if overflow occurred.
    #[inline]
    pub fn checked_add(&self, other: Decimal) -> Option<Decimal> {
        if self.negative != other.negative {
            if other.negative {
                self.sub_internal(&other, self.negative)
            } else {
                other.sub_internal(self, other.negative)
            }
        } else {
            self.add_internal(&other, self.negative)
        }
    }

    /// Subtract one decimal from another,
    /// returning `None` if overflow occurred.
    #[inline]
    pub fn checked_sub(&self, other: Decimal) -> Option<Decimal> {
        if self.negative != other.negative {
            self.add_internal(&other, self.negative)
        } else if self.negative {
            other.sub_internal(self, !self.negative)
        } else {
            self.sub_internal(&other, self.negative)
        }
    }

    /// Calculate the product of two decimals,
    /// returning `None` if overflow occurred.
    #[inline]
    pub fn checked_mul(&self, other: Decimal) -> Option<Decimal> {
        if self.is_zero() || other.is_zero() {
            return Some(Decimal::ZERO);
        }

        let scale = self.scale + other.scale;
        let negative = self.negative ^ other.negative;
        let int_val = U256::mul128(self.int_val, other.int_val);

        if !int_val.is_decimal_overflowed() && scale == 0 {
            Some(unsafe { Decimal::from_parts_unchecked(int_val.low(), 0, negative) })
        } else {
            Decimal::adjust_scale(int_val, scale, negative)
        }
    }

    /// Checked decimal division.
    /// Computes `self / other`, returning `None` if `other == 0` or the division results in overflow.
    #[inline]
    pub fn checked_div(&self, other: Decimal) -> Option<Decimal> {
        if other.is_zero() {
            return None;
        }

        if self.is_zero() {
            return Some(Decimal::ZERO);
        }

        let other_precision = other.precision();
        let self_precision = self.precision();

        let (self_int_val, shift_precision) = if other_precision > self_precision {
            let p = MAX_PRECISION + (other_precision - self_precision) as u32;
            (POWERS_10[p as usize] * self.int_val, other_precision - self_precision)
        } else {
            (U256::mul128(self.int_val, POWERS_10[MAX_PRECISION as usize].low()), 0)
        };

        let negative = self.negative ^ other.negative;
        let int_val = self_int_val.div128_round(other.int_val);
        let scale = self.scale - other.scale + MAX_PRECISION as i16 + shift_precision as i16;

        Decimal::adjust_scale(int_val, scale, negative)
    }

    /// Checked decimal remainder.
    /// Computes `self % other`, returning None if rhs == 0 or the division results in overflow.
    #[inline]
    pub fn checked_rem(&self, other: Decimal) -> Option<Decimal> {
        if other.is_zero() {
            return None;
        }

        if self.is_zero() {
            return Some(Decimal::ZERO);
        }

        if self.scale == other.scale {
            let rem = self.int_val % other.int_val;
            return Some(unsafe { Decimal::from_parts_unchecked(rem, self.scale, self.negative) });
        }

        if self.scale < other.scale {
            let e = other.scale - self.scale;
            debug_assert!(e > 0);

            if e as u32 > MAX_PRECISION {
                let (self_int_val, scale) = if e as usize >= POWERS_10.len() {
                    (
                        POWERS_10[MAX_PRECISION as usize] * self.int_val,
                        self.scale + MAX_PRECISION as i16,
                    )
                } else {
                    (
                        POWERS_10[(other.scale - self.scale) as usize] * self.int_val,
                        other.scale,
                    )
                };

                let (_int_val, rem) = self_int_val.div_rem(other.int_val);

                return Some(Decimal {
                    int_val: rem.low(),
                    scale,
                    negative: self.negative,
                });
            }

            let self_int_val = U256::mul128(self.int_val, POWERS_10[e as usize].low());
            let rem = self_int_val % other.int_val;

            Decimal::adjust_scale(rem, other.scale, self.negative)
        } else {
            let e = self.scale - other.scale;
            debug_assert!(e > 0);
            if e as u32 > MAX_PRECISION {
                return Some(*self);
            }

            let other_int_val = U256::mul128(other.int_val, POWERS_10[e as usize].low());
            let rem = self.int_val % other_int_val;

            Decimal::adjust_scale(rem, self.scale, self.negative)
        }
    }

    /// Computes the square root of a decimal,
    /// returning None if `self` is negative or the results in overflow.
    #[inline]
    pub fn sqrt(&self) -> Option<Decimal> {
        if self.negative {
            return None;
        }

        if self.is_zero() {
            return Some(Decimal::ZERO);
        }

        let mut result = self.checked_mul(Decimal::ZERO_POINT_FIVE)?;
        let mut last = result;

        loop {
            let val = self.checked_div(result)?.normalize();
            result = result.checked_add(val)?;
            result = result.checked_mul(Decimal::ZERO_POINT_FIVE)?;

            if result == last {
                break;
            }

            last = result;
        }

        Some(result)
    }
}

impl fmt::Display for Decimal {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = Buf::new();
        self.fmt_internal(false, f.precision(), &mut buf);
        let str = unsafe { std::str::from_utf8_unchecked(buf.as_slice()) };
        f.pad_integral(self.is_sign_positive(), "", str)
    }
}

impl Default for Decimal {
    #[inline]
    fn default() -> Self {
        Decimal::ZERO
    }
}

impl PartialEq for Decimal {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialEq<&Decimal> for Decimal {
    #[inline]
    fn eq(&self, other: &&Decimal) -> bool {
        self.eq(*other)
    }
}

impl PartialEq<Decimal> for &Decimal {
    #[inline]
    fn eq(&self, other: &Decimal) -> bool {
        (*self).eq(other)
    }
}

impl PartialOrd for Decimal {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<&Decimal> for Decimal {
    #[inline]
    fn partial_cmp(&self, other: &&Decimal) -> Option<Ordering> {
        self.partial_cmp(*other)
    }
}

impl PartialOrd<Decimal> for &Decimal {
    #[inline]
    fn partial_cmp(&self, other: &Decimal) -> Option<Ordering> {
        (*self).partial_cmp(other)
    }
}

impl Ord for Decimal {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // sign is different
        if self.negative != other.negative {
            return if self.negative {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        let (left, right) = if self.negative {
            // both are negative, so reverse cmp
            debug_assert!(other.negative);
            (other, self)
        } else {
            (self, other)
        };

        if left.is_zero() {
            return if right.is_zero() {
                Ordering::Equal
            } else {
                Ordering::Less
            };
        } else if right.is_zero() {
            return Ordering::Greater;
        }

        if left.scale == right.scale {
            // fast path for same scale
            return left.int_val.cmp(&right.int_val);
        }

        if left.scale < right.scale {
            left.rescale_cmp(right)
        } else {
            right.rescale_cmp(left).reverse()
        }
    }
}

impl Hash for Decimal {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let n = self.normalize();
        n.int_val.hash(state);
        n.scale.hash(state);
        n.negative.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_internal() {
        fn assert(
            int_val: u128,
            scale: i16,
            negative: bool,
            append_sign: bool,
            precision: Option<usize>,
            expected: &str,
        ) {
            let dec = Decimal::from_parts(int_val, scale, negative).unwrap();
            let mut buf = Buf::new();
            dec.fmt_internal(append_sign, precision, &mut buf);
            let str = unsafe { std::str::from_utf8_unchecked(buf.as_slice()) };
            assert_eq!(str, expected);
        }

        assert(128, 0, false, false, None, "128");
        assert(128, -2, true, true, None, "-12800");
        assert(128, 4, true, true, None, "-0.0128");
        assert(128, 2, true, false, None, "1.28");
        assert(12856, 4, true, false, None, "1.2856");
        assert(12856, 4, true, false, Some(2), "1.29");
        assert(12856, 4, true, false, Some(6), "1.285600");
        assert(1285600, 6, false, false, None, "1.2856");
    }

    #[test]
    fn test_display() {
        macro_rules! assert_display {
            ($num: expr, $scale: expr, $negative: expr, $fmt: expr,$expected: expr) => {{
                let dec = Decimal::from_parts($num, $scale, $negative).unwrap();
                let str = format!($fmt, dec);
                assert_eq!(str, $expected);
            }};
        }

        assert_display!(0, -1, false, "{}", "0");
        assert_display!(1, 0, false, "{}", "1");
        assert_display!(1, 1, false, "{}", "0.1");
        assert_display!(1, -1, false, "{}", "10");
        assert_display!(10, 0, false, "{}", "10");
        assert_display!(10, 1, false, "{}", "1");
        assert_display!(10, -1, false, "{}", "100");
        assert_display!(128, 0, false, "{}", "128");
        assert_display!(128, -2, true, "{}", "-12800");
        assert_display!(128, 4, true, "{}", "-0.0128");
        assert_display!(128, 2, true, "{}", "-1.28");
        assert_display!(12800, 1, false, "{}", "1280");
        assert_display!(12800, 2, false, "{}", "128");
        assert_display!(12800, 3, false, "{}", "12.8");
        assert_display!(12856, 4, true, "{}", "-1.2856");
        assert_display!(12856, 4, true, "{:.2}", "-1.29");
        assert_display!(12856, 4, true, "{:.6}", "-1.285600");
        assert_display!(12856, 0, true, "{:.6}", "-12856.000000");
        assert_display!(1285600, 6, false, "{}", "1.2856");
        assert_display!(u64::MAX as u128, 0, false, "{}", u64::MAX.to_string());
        assert_display!(101, -98, false, "{:.10}", "10100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0000000000");
        assert_display!(101, 98, false, "{:.10}", "0.0000000000");
    }

    #[test]
    fn test_precision() {
        fn assert_precision(val: &str, expected: u8) {
            let dec = val.parse::<Decimal>().unwrap();
            assert_eq!(dec.precision(), expected);
        }

        assert_precision("0.0", 1);
        assert_precision("1", 1);
        assert_precision("10", 2);
        assert_precision("1.230", 3);
        assert_precision("123456123456", 12);
        assert_precision("123456.123456", 12);
        assert_precision("-123456.123456", 12);
        assert_precision("99999999999999999999999999999999999999", 38);
    }

    #[test]
    fn test_encoding() {
        fn assert_encoding(num: &str) {
            let num = num.parse::<Decimal>().unwrap();
            let mut buf = Vec::new();
            let size = num.compact_encode(&mut buf).unwrap();
            assert_eq!(buf.len(), size);
            let decoded_num = Decimal::decode(&buf);
            assert_eq!(decoded_num, num);
        }

        assert_encoding("0");
        assert_encoding("255");
        assert_encoding("-255");
        assert_encoding("65535");
        assert_encoding("-65535");
        assert_encoding("4294967295");
        assert_encoding("-4294967295");
        assert_encoding("18446744073709551615");
        assert_encoding("-18446744073709551615");
        assert_encoding("99999999999999999999999999999999999999");
        assert_encoding("-99999999999999999999999999999999999999");
        assert_encoding("184467440.73709551615");
        assert_encoding("-184467440.73709551615");
    }

    #[test]
    fn test_cmp() {
        macro_rules! assert_cmp {
            ($left: expr, $cmp: tt, $right: expr) => {{
                let l = $left.parse::<Decimal>().unwrap();
                let r = $right.parse::<Decimal>().unwrap();
                assert!(l $cmp r, "{} {} {}", l, stringify!($cmp),r);
            }};
        }

        assert_cmp!("0", ==, "0");

        assert_cmp!("-1", <, "1");
        assert_cmp!("1", >, "-1");

        assert_cmp!("1.1", ==, "1.1");
        assert_cmp!("1.2", >, "1.1");
        assert_cmp!("-1.2", <, "1.1");
        assert_cmp!("1.1", >, "-1.2");

        assert_cmp!("1", <, "1e39");
        assert_cmp!("1", >, "1e-39");
        assert_cmp!("1.0e-100", >=, "1.0e-101");
        assert_cmp!("1.0e-101", <=, "1.0e-100");
        assert_cmp!("1.0e-100", !=, "1.0e-101");

        assert_cmp!("1.12", <, "1.2");
        assert_cmp!("1.2", >, "1.12");
        assert_cmp!("-1.2", <, "-1.12");
        assert_cmp!("-1.12", >, "-1.2");
        assert_cmp!("-1.12", <, "1.2");
        assert_cmp!("1.12", >, "-1.2");

        assert_cmp!("0.000000001", <,"100000000");
        assert_cmp!("100000000", >, "0.000000001");

        assert_cmp!(
            "9999999999999999999999999999999999999.9", >, "9.9999999999999999999999999999999999999"
        );
        assert_cmp!(
            "9.9999999999999999999999999999999999999", >, "0"
        );
        assert_cmp!(
            "9.9999999999999999999999999999999999999", >, "1"
        );
        assert_cmp!(
            "-9999999999999999999999999999999999999.9", <, "-9.9999999999999999999999999999999999999"
        );
        assert_cmp!(
            "-9.9999999999999999999999999999999999999", <, "0"
        );
        assert_cmp!(
            "-9.9999999999999999999999999999999999999", <, "1"
        );
        assert_cmp!("4703178999618078116505370421100e39", >, "0");
        assert_cmp!("4703178999618078116505370421100e-39", >, "0");
        assert_cmp!("-4703178999618078116505370421100e39", <, "0");
        assert_cmp!("-4703178999618078116505370421100e-39", <, "0");
        assert_cmp!("0", <, "4703178999618078116505370421100e39");
        assert_cmp!("0", <, "4703178999618078116505370421100e-39");
        assert_cmp!("0", >, "-4703178999618078116505370421100e39");
        assert_cmp!("0", >, "-4703178999618078116505370421100e-39");
    }

    #[test]
    fn test_abs() {
        fn assert_abs(val: &str, expected: &str) {
            let abs_val = val.parse::<Decimal>().unwrap().abs();
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(abs_val, expected);
        }

        assert_abs("0.0", "0");
        assert_abs("123456.123456", "123456.123456");
        assert_abs("-123456.123456", "123456.123456");
    }

    #[test]
    fn test_trunc() {
        fn assert_trunc(val: &str, scale: i16, expected: &str) {
            let decimal = val.parse::<Decimal>().unwrap().trunc(scale);
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }

        assert_trunc("0", -1, "0");
        assert_trunc("123456", 0, "123456");
        assert_trunc("123456.123456", 6, "123456.123456");
        assert_trunc("123456.123456", 5, "123456.12345");
        assert_trunc("123456.123456", 4, "123456.1234");
        assert_trunc("123456.123456", 3, "123456.123");
        assert_trunc("123456.123456", 2, "123456.12");
        assert_trunc("123456.123456", 1, "123456.1");
        assert_trunc("123456.123456", 0, "123456");
        assert_trunc("123456.123456", -1, "123450");
        assert_trunc("123456.123456", -2, "123400");
        assert_trunc("123456.123456", -3, "123000");
        assert_trunc("123456.123456", -4, "120000");
        assert_trunc("123456.123456", -5, "100000");
        assert_trunc("9999.9", 1, "9999.9");
        assert_trunc("9999.9", -2, "9900");
        assert_trunc("9999.9", -4, "0");
        assert_trunc("1e126", 0, "1e126");
        assert_trunc("1e126", -126, "1e126");
        assert_trunc("1e-130", 0, "0");
    }

    #[test]
    fn test_round() {
        fn assert_round(val: &str, scale: i16, expected: &str) {
            let decimal = val.parse::<Decimal>().unwrap().round(scale);
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }

        assert_round("0", -1, "0");
        assert_round("123456", 0, "123456");
        assert_round("123456.123456", 6, "123456.123456");
        assert_round("123456.123456", 5, "123456.12346");
        assert_round("123456.123456", 4, "123456.1235");
        assert_round("123456.123456", 3, "123456.123");
        assert_round("123456.123456", 2, "123456.12");
        assert_round("123456.123456", 1, "123456.1");
        assert_round("123456.123456", 0, "123456");
        assert_round("123456.123456", -1, "123460");
        assert_round("123456.123456", -2, "123500");
        assert_round("123456.123456", -3, "123000");
        assert_round("123456.123456", -4, "120000");
        assert_round("123456.123456", -5, "100000");
        assert_round("9999.9", 1, "9999.9");
        assert_round("9999.9", -2, "10000");
        assert_round("9999.9", -4, "10000");
    }

    #[test]
    fn test_round_with_precision() {
        fn assert(val: &str, precision: u8, scale: i16, expected: &str) {
            let mut decimal = val.parse::<Decimal>().unwrap();
            let overflowed = decimal.round_with_precision(precision, scale);
            assert!(!overflowed);
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }

        fn assert_overflow(val: &str, precision: u8, scale: i16) {
            let mut decimal = val.parse::<Decimal>().unwrap();
            let overflowed = decimal.round_with_precision(precision, scale);
            assert!(overflowed);
        }

        assert_overflow("123456", 5, 0);
        assert_overflow("123456", 5, 1);
        assert_overflow("123456", 6, 1);
        assert_overflow("123.456", 6, 4);
        assert("123456", 5, -1, "123460");
        assert("123456", 5, -5, "100000");
        assert("123456", 5, -6, "0");
        assert("123456", 6, 0, "123456");
        assert("123456", 6, -1, "123460");
        assert("123.456", 6, 0, "123");
        assert("123.456", 6, 1, "123.5");
        assert("123.456", 6, 3, "123.456");
        assert("123.456", 6, -1, "120");
        assert("123.456", 6, -2, "100");
        assert("123.456", 6, -3, "0");
        assert("123.456", 6, -4, "0");
    }

    #[test]
    fn test_normalize() {
        fn assert_normalize(val: (u128, i16), expected: (u128, i16)) {
            let left = Decimal::from_parts(val.0, val.1, false).unwrap();
            let right = Decimal::from_parts(expected.0, expected.1, false).unwrap();
            assert_eq!(left, right);
            let normal = left.normalize();
            assert_eq!((normal.int_val, normal.scale), expected);
        }

        assert_normalize((12300, MAX_SCALE), (123, MAX_SCALE - 2));
        assert_normalize((12300, 2), (123, 0));
        assert_normalize((1230, 0), (1230, 0));
        assert_normalize((12300, -2), (1230000, 0));
        assert_normalize(
            (9_9999_9999_9999_9999_9999_9999_9999_9999_9999_u128, -2),
            (99_9999_9999_9999_9999_9999_9999_9999_9999_9990_u128, -1),
        );
        assert_normalize((12300, MIN_SCALE + 1), (12300000000000000000000000000000000000, -92));
    }

    #[test]
    fn test_hash() {
        use std::collections::hash_map::DefaultHasher;

        let d1 = Decimal::from_parts(12345, 3, false).unwrap();
        let d2 = Decimal::from_parts(123450, 4, false).unwrap();

        let mut hash1 = DefaultHasher::new();
        let mut hash2 = DefaultHasher::new();

        d1.hash(&mut hash1);
        d2.hash(&mut hash2);

        assert_eq!(hash1.finish(), hash2.finish());
    }

    #[test]
    fn test_sqrt() {
        fn assert_sqrt(val: &str, expected: &str) {
            let num = val.parse::<Decimal>().unwrap();
            let expected = expected.parse::<Decimal>().unwrap();
            let result = num.sqrt().unwrap();
            assert_eq!(result, expected);
        }

        assert_sqrt("0", "0");
        assert_sqrt("0.00000", "0");
        assert_sqrt("1", "1");
        assert_sqrt("1.001", "1.0004998750624609648232582877001097531");
        assert_sqrt("1.44", "1.2");
        assert_sqrt("2", "1.4142135623730950488016887242096980786");
        assert_sqrt("100", "10");
        assert_sqrt("49", "7");
        assert_sqrt("0.25", "0.5");
        assert_sqrt("0.0152399025", "0.12345");
        assert_sqrt("152399025", "12345");
        assert_sqrt("0.00400", "0.063245553203367586639977870888654370675");
        assert_sqrt("0.1", "0.31622776601683793319988935444327185337");
        assert_sqrt("2", "1.4142135623730950488016887242096980786");
        assert_sqrt("125348", "354.04519485512015631084871931761013143");
        assert_sqrt(
            "18446744073709551616.1099511",
            "4294967296.0000000000127999926917254925",
        );
        assert_sqrt(
            "3.1415926535897931159979634685441851615",
            "1.7724538509055159927515191031392484393",
        );
        assert_sqrt(
            "0.000000000089793115997963468544185161590576171875",
            "0.0000094759229628550415175617837401442254225",
        );
        assert_sqrt(
            "0.71777001097629639227453423431674136248",
            "0.84721308475276536670429805177990207040",
        );
        assert_sqrt(
            "0.012345679012345679012345679012345679012",
            "0.11111111111111111111111111111111111111",
        );
        assert_sqrt(
            "0.11088900000000000000000000000000000444",
            "0.33300000000000000000000000000000000667",
        );
        assert_sqrt(
            "17014118346046923173168730371588410572",
            "4124817371235594858.7903221175243613899",
        );
        assert_sqrt(
            "0.17014118346046923173168730371588410572",
            "0.41248173712355948587903221175243613899",
        );
        assert_sqrt("1e100", "1e50");
        assert_sqrt("1.01e100", "1.0049875621120890270219264912759576187e50");
        assert_sqrt("1e-100", "1e-50");
        assert_sqrt("1.01e-100", "1.0049875621120890270219264912759576187e-50");
    }
}

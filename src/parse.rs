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

//! Decimal parsing utilities.

use crate::convert::MAX_I128_REPR;
use crate::decimal::{MAX_PRECISION, MAX_SCALE, MIN_SCALE};
use crate::error::DecimalParseError;
use crate::Decimal;
use std::convert::TryInto;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Sign {
    Positive,
    Negative,
}

/// The interesting parts of a decimal string.
#[derive(Debug)]
struct Parts<'a> {
    pub sign: Sign,
    pub integral: &'a [u8],
    pub fractional: &'a [u8],
    pub exp: i16,
}

/// Splits a decimal string bytes into sign and the rest, without inspecting or validating the rest.
#[inline]
fn extract_sign(s: &[u8]) -> (Sign, &[u8]) {
    match s.first() {
        Some(b'+') => (Sign::Positive, &s[1..]),
        Some(b'-') => (Sign::Negative, &s[1..]),
        _ => (Sign::Positive, s),
    }
}

/// Carves off decimal digits up to the first non-digit character.
#[inline]
fn eat_digits(s: &[u8]) -> (&[u8], &[u8]) {
    let i = s.iter().take_while(|&i| i.is_ascii_digit()).count();
    (&s[..i], &s[i..])
}

/// Extracts exponent, if any.
fn extract_exponent(s: &[u8]) -> Result<(i16, &[u8]), DecimalParseError> {
    let (sign, s) = extract_sign(s);
    let (mut number, s) = eat_digits(s);

    if number.is_empty() {
        return Err(DecimalParseError::Invalid);
    }

    while number.first() == Some(&b'0') {
        number = &number[1..];
    }

    if number.len() > 3 {
        return match sign {
            Sign::Positive => Err(DecimalParseError::Overflow),
            Sign::Negative => Err(DecimalParseError::Underflow),
        };
    }

    let exp = {
        let mut result: i16 = 0;
        for &n in number {
            result = result * 10 + (n - b'0') as i16;
        }
        match sign {
            Sign::Positive => result,
            Sign::Negative => -result,
        }
    };

    Ok((exp, s))
}

/// Checks if the input string is a valid decimal and if so, locate the integral
/// part, the fractional part, and the exponent in it.
fn parse_decimal(s: &[u8]) -> Result<(Parts, &[u8]), DecimalParseError> {
    let (sign, s) = extract_sign(s);

    if s.is_empty() {
        return Err(DecimalParseError::Invalid);
    }

    let (mut integral, s) = eat_digits(s);

    while integral.first() == Some(&b'0') && integral.len() > 1 {
        integral = &integral[1..];
    }

    let (fractional, exp, s) = match s.first() {
        Some(&b'e') | Some(&b'E') => {
            if integral.is_empty() {
                return Err(DecimalParseError::Invalid);
            }

            let (exp, s) = extract_exponent(&s[1..])?;
            (&b""[..], exp, s)
        }
        Some(&b'.') => {
            let (mut fractional, s) = eat_digits(&s[1..]);
            if integral.is_empty() && fractional.is_empty() {
                return Err(DecimalParseError::Invalid);
            }

            while fractional.last() == Some(&b'0') {
                fractional = &fractional[0..fractional.len() - 1];
            }

            match s.first() {
                Some(&b'e') | Some(&b'E') => {
                    let (exp, s) = extract_exponent(&s[1..])?;
                    (fractional, exp, s)
                }
                _ => (fractional, 0, s),
            }
        }
        _ => {
            if integral.is_empty() {
                return Err(DecimalParseError::Invalid);
            }

            (&b""[..], 0, s)
        }
    };

    Ok((
        Parts {
            sign,
            integral,
            fractional,
            exp,
        },
        s,
    ))
}

/// Carves off whitespaces up to the first non-whitespace character.
#[inline]
fn eat_whitespaces(s: &[u8]) -> &[u8] {
    let i = s.iter().take_while(|&i| i.is_ascii_whitespace()).count();
    &s[i..]
}

/// Extracts `NaN` value.
#[inline]
fn extract_nan(s: &[u8]) -> (bool, &[u8]) {
    if s.len() < 3 {
        (false, s)
    } else {
        let mut buf: [u8; 3] = s[0..3].try_into().unwrap();
        buf.make_ascii_lowercase();
        if &buf == b"nan" {
            (true, &s[3..])
        } else {
            (false, s)
        }
    }
}

/// Parses a string bytes and put the number into this variable.
///
/// This function does not handle leading or trailing spaces, and it doesn't
/// accept `NaN` either. It returns the remaining string bytes so that caller can
/// check for trailing spaces/garbage if deemed necessary.
#[inline]
fn parse_str(s: &[u8]) -> Result<(Decimal, &[u8]), DecimalParseError> {
    let (
        Parts {
            sign,
            integral,
            fractional,
            exp,
        },
        s,
    ) = parse_decimal(s)?;

    let mut integral = integral;
    let mut fractional = fractional;
    let mut scale = -exp;

    let mut carry = false;
    const MAX_PRECISION_USIZE: usize = MAX_PRECISION as usize;

    // normalized_exp is the exponent of a number with the format `0.{fractional}E{exponent}`, and the first digit of `fractional` is not 0.
    // Suppose `a = 123.456e12`, convert `a` to the format above and get `0.123456e15`, then the normalized_exp of a is 15.
    let mut normalized_exp = exp;

    if integral == b"0" {
        // fractional only
        let zero_count = fractional.iter().take_while(|i| **i == b'0').count();
        normalized_exp -= zero_count as i16;

        let max_fractional_precision = MAX_PRECISION_USIZE + zero_count;
        if fractional.len() > max_fractional_precision {
            carry = fractional[max_fractional_precision] > b'4';
            fractional = &fractional[0..max_fractional_precision];
        }

        debug_assert!(fractional.len() <= max_fractional_precision);
    } else {
        let int_len = integral.len() as i16;
        normalized_exp += int_len;

        if int_len > MAX_PRECISION_USIZE as i16 {
            carry = integral[MAX_PRECISION_USIZE] > b'4';
            scale -= int_len - MAX_PRECISION_USIZE as i16;

            integral = &integral[0..MAX_PRECISION_USIZE];
            fractional = &[];
        } else {
            let max_fractional_precision = MAX_PRECISION_USIZE - int_len as usize;
            if fractional.len() > max_fractional_precision {
                carry = fractional[max_fractional_precision] > b'4';
                fractional = &fractional[0..max_fractional_precision];
            }

            debug_assert!(fractional.len() <= max_fractional_precision);
        }
    };

    let mut int = 0u128;
    for &i in integral {
        int = int * 10 + (i - b'0') as u128;
    }
    for &i in fractional {
        int = int * 10 + (i - b'0') as u128;
    }
    // So far, `int` precision does not exceed MAX_PRECISION.

    int += carry as u128;
    if int > MAX_I128_REPR as u128 {
        normalized_exp += 1;
        int /= 10;
        scale -= 1;
    }

    if normalized_exp <= -MAX_SCALE {
        return Err(DecimalParseError::Underflow);
    }
    if normalized_exp > -MIN_SCALE {
        return Err(DecimalParseError::Overflow);
    }

    let negative = if int != 0 { sign == Sign::Negative } else { false };

    scale += fractional.len() as i16;
    Ok((unsafe { Decimal::from_parts_unchecked(int, scale, negative) }, s))
}

/// Parses a string slice and creates a decimal.
///
/// This function handles leading or trailing spaces, and it
/// accepts `NaN` either.
#[inline]
fn from_str(s: &str) -> Result<Decimal, DecimalParseError> {
    let s = s.as_bytes();
    let s = eat_whitespaces(s);
    if s.is_empty() {
        return Err(DecimalParseError::Empty);
    }

    let (is_nan, s) = extract_nan(s);

    if is_nan {
        Err(DecimalParseError::Invalid)
    } else {
        let (n, s) = parse_str(s)?;

        if s.iter().any(|n| !n.is_ascii_whitespace()) {
            return Err(DecimalParseError::Invalid);
        }

        Ok(n)
    }
}

impl FromStr for Decimal {
    type Err = DecimalParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_parse_empty<S: AsRef<str>>(s: S) {
        let result = s.as_ref().parse::<Decimal>();
        assert_eq!(result.unwrap_err(), DecimalParseError::Empty);
    }

    fn assert_parse_invalid<S: AsRef<str>>(s: S) {
        let result = s.as_ref().parse::<Decimal>();
        assert_eq!(result.unwrap_err(), DecimalParseError::Invalid);
    }

    fn assert_parse_overflow<S: AsRef<str>>(s: S) {
        let result = s.as_ref().parse::<Decimal>();
        assert_eq!(result.unwrap_err(), DecimalParseError::Overflow);
    }

    fn assert_parse_underflow<S: AsRef<str>>(s: S) {
        let result = s.as_ref().parse::<Decimal>();
        assert_eq!(result.unwrap_err(), DecimalParseError::Underflow);
    }

    #[test]
    fn test_parse_error() {
        assert_parse_empty("");
        assert_parse_empty("   ");
        assert_parse_invalid("-");
        assert_parse_invalid("   -   ");
        assert_parse_invalid("-.");
        assert_parse_invalid("- 1");
        assert_parse_invalid("-NaN");
        assert_parse_invalid("NaN.");
        assert_parse_invalid("NaN1");
        assert_parse_invalid("   NaN   .   ");
        assert_parse_invalid("   NaN   1   ");
        assert_parse_invalid(".");
        assert_parse_invalid("   .   ");
        assert_parse_invalid("e");
        assert_parse_invalid("   e   ");
        assert_parse_invalid("-e");
        assert_parse_invalid("-1e");
        assert_parse_invalid("1e1.1");
        assert_parse_invalid("-1 e1");
        assert_parse_invalid("   x   ");
        assert_parse_overflow("1e1000");
        assert_parse_overflow("1e100000");
        assert_parse_overflow("1e127");
        assert_parse_underflow("1e-131");
        assert_parse_underflow("1e-1000");
        assert_parse_underflow("1e-100000");
    }

    fn assert_parse<S: AsRef<str>, V: AsRef<str>>(s: S, expected: V) {
        let decimal = s.as_ref().parse::<Decimal>().unwrap();
        assert_eq!(decimal.to_string(), expected.as_ref());
    }

    #[test]
    fn test_parse_valid() {
        // Integer
        assert_parse("0", "0");
        assert_parse("-0", "0");
        assert_parse("   -0   ", "0");
        assert_parse("00000.", "0");
        assert_parse("-00000.", "0");
        assert_parse("128", "128");
        assert_parse("-128", "-128");
        assert_parse("65536", "65536");
        assert_parse("-65536", "-65536");
        assert_parse("4294967296", "4294967296");
        assert_parse("-4294967296", "-4294967296");
        assert_parse("18446744073709551616", "18446744073709551616");
        assert_parse("-18446744073709551616", "-18446744073709551616");
        assert_parse(
            "99999999999999999999999999999999999999",
            "99999999999999999999999999999999999999",
        );
        assert_parse(
            "0099999999999999999999999999999999999999",
            "99999999999999999999999999999999999999",
        );
        assert_parse(
            "-99999999999999999999999999999999999999",
            "-99999999999999999999999999999999999999",
        );
        assert_parse("000000000123", "123");
        assert_parse("-000000000123", "-123");
        assert_parse(
            "170141183460469231713240559642175554110",
            "170141183460469231713240559642175554110",
        );
        assert_parse(
            "999999999999999999999999999999999999990000000000",
            "999999999999999999999999999999999999990000000000",
        );

        // Floating-point number
        assert_parse("0.0", "0");
        assert_parse("-0.0", "0");
        assert_parse("   -0.0   ", "0");
        assert_parse(".0", "0");
        assert_parse(".00000", "0");
        assert_parse("-.0", "0");
        assert_parse("-.00000", "0");
        assert_parse("128.128", "128.128");
        assert_parse("-128.128", "-128.128");
        assert_parse("65536.65536", "65536.65536");
        assert_parse("-65536.65536", "-65536.65536");
        assert_parse("4294967296.4294967296", "4294967296.4294967296");
        assert_parse("-4294967296.4294967296", "-4294967296.4294967296");
        assert_parse(
            "9999999999999999999.9999999999999999999",
            "9999999999999999999.9999999999999999999",
        );
        assert_parse(
            "-9999999999999999999.9999999999999999999",
            "-9999999999999999999.9999999999999999999",
        );
        assert_parse("000000000123.000000000123", "123.000000000123");
        assert_parse("-000000000123.000000000123", "-123.000000000123");
        assert_parse(
            "00.000000000000000000000000000000000000123",
            "0.000000000000000000000000000000000000123",
        );
        assert_parse("00.000000000000000000000000000000000000123e-87", "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000123");
        assert_parse("99999999999999999999999999999999999999500000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

        // Scientific notation
        assert_parse("0e0", "0");
        assert_parse("-0E-0", "0");
        assert_parse("0000000000E0000000000", "0");
        assert_parse("-0000000000E-0000000000", "0");
        assert_parse("00000000001e0000000000", "1");
        assert_parse("-00000000001e-0000000000", "-1");
        assert_parse("00000000001e00000000001", "10");
        assert_parse("-00000000001e-00000000001", "-0.1");
        assert_parse("1e10", "10000000000");
        assert_parse("-1e-10", "-0.0000000001");
        assert_parse("0000001.23456000e3", "1234.56");
        assert_parse("-0000001.23456000E-3", "-0.00123456");
    }

    #[test]
    fn test_parse_boundary() {
        assert_parse("100E-131", "0.00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100");
        assert_parse("0.000012345E130", "123450000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        assert_parse("4.94065645841247E-126", "0.00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000494065645841247");
        assert_parse("1234.94065645841247E-126", "0.00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000123494065645841247");
        assert_parse("12345678987654321999999E-132", "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678987654321999999");
        assert_parse("10000000000000000000000000000000000000e88", "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        assert_parse("0.999999999999999999999999999999999999995e-130", "0.00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000");
        assert_parse_underflow("0.999999999999999999999999999999999999995e-131");
        assert_parse_overflow("999999999999999999999999999999999999995000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_parse_over_precision_but_valid() {
        // integer only
        assert_parse(
            "999999999999999999999999999999999999999",
            "1000000000000000000000000000000000000000",
        );
        assert_parse(
            "900719925474099290071992547409929007112123123123123",
            "900719925474099290071992547409929007110000000000000",
        );

        // fractional only
        assert_parse(
            "0.123123123123123135555555555555555555555555555555",
            "0.12312312312312313555555555555555555556",
        );
        assert_parse(
            "0.0000000123123123123123135555555555555555555555555555555",
            "0.000000012312312312312313555555555555555555556",
        );
        assert_parse(
            "0.0000000123123123123123135555555555555515555555555555555",
            "0.000000012312312312312313555555555555551555556",
        );
        assert_parse(
            "0.0000000123123123123123135555555555555565555551555555555",
            "0.000000012312312312312313555555555555556555555",
        );

        // integer over precision
        assert_parse(
            "1231231231231231231231231255555555555555555555.123",
            "1231231231231231231231231255555555555600000000",
        );

        // integer + fractional over precision
        assert_parse(
            "123123.5555555555555555555555555555555555555555",
            "123123.55555555555555555555555555555556",
        );

        assert_parse_overflow("90071992547409929007199254740992900711212312312312312312312312312311111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111");
    }
}

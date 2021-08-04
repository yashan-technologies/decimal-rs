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

//! Ops implementation.

use crate::decimal::Decimal;
use std::convert::TryFrom;
use std::iter::{Product, Sum};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

impl Neg for Decimal {
    type Output = Decimal;

    #[inline]
    fn neg(mut self) -> Self::Output {
        if !self.is_zero() {
            self.negative = !self.negative;
        }
        self
    }
}

impl Neg for &'_ Decimal {
    type Output = Decimal;

    #[inline]
    fn neg(self) -> Self::Output {
        if !self.is_zero() {
            unsafe { Decimal::from_parts_unchecked(self.int_val, self.scale, !self.negative) }
        } else {
            Decimal::ZERO
        }
    }
}

impl Add<Decimal> for &'_ Decimal {
    type Output = Decimal;

    #[inline(always)]
    fn add(self, other: Decimal) -> Self::Output {
        match self.checked_add(other) {
            Some(sum) => sum,
            None => panic!("Addition overflowed"),
        }
    }
}

impl AddAssign for Decimal {
    #[inline(always)]
    fn add_assign(&mut self, other: Decimal) {
        let result = self.add(other);
        *self = result;
    }
}

impl Sub<Decimal> for &'_ Decimal {
    type Output = Decimal;

    #[inline(always)]
    fn sub(self, other: Decimal) -> Decimal {
        match self.checked_sub(other) {
            Some(diff) => diff,
            None => panic!("Subtraction overflowed"),
        }
    }
}

impl SubAssign for Decimal {
    #[inline(always)]
    fn sub_assign(&mut self, other: Decimal) {
        let result = self.sub(other);
        *self = result;
    }
}

impl Mul<Decimal> for &'_ Decimal {
    type Output = Decimal;

    #[inline(always)]
    fn mul(self, other: Decimal) -> Decimal {
        match self.checked_mul(other) {
            Some(prod) => prod,
            None => panic!("Multiplication overflowed"),
        }
    }
}

impl MulAssign for Decimal {
    #[inline(always)]
    fn mul_assign(&mut self, other: Decimal) {
        let result = self.mul(other);
        *self = result;
    }
}

impl Div<Decimal> for &'_ Decimal {
    type Output = Decimal;

    #[inline(always)]
    fn div(self, other: Decimal) -> Decimal {
        match self.checked_div(other) {
            Some(quot) => quot,
            None => panic!("Division by zero or overflowed"),
        }
    }
}

impl DivAssign for Decimal {
    #[inline(always)]
    fn div_assign(&mut self, other: Decimal) {
        let result = self.div(other);
        *self = result;
    }
}

impl Rem<Decimal> for &Decimal {
    type Output = Decimal;

    #[inline(always)]
    fn rem(self, other: Decimal) -> Decimal {
        match self.checked_rem(other) {
            Some(rem) => rem,
            None => panic!("Division by zero or overflowed"),
        }
    }
}

impl RemAssign for Decimal {
    #[inline(always)]
    fn rem_assign(&mut self, other: Decimal) {
        let result = self.rem(other);
        *self = result;
    }
}

impl Sum for Decimal {
    #[inline(always)]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Decimal::ZERO, Add::add)
    }
}

impl Product for Decimal {
    #[inline(always)]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Decimal::ONE, Mul::mul)
    }
}

macro_rules! impl_arith_with_num {
    ($op: ident { $method: ident } $int: ty) => {
        impl $op<$int> for Decimal {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: $int) -> Self::Output {
                self.$method(Decimal::from(other))
            }
        }

        impl $op<$int> for &'_ Decimal {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: $int) -> Self::Output {
                self.$method(Decimal::from(other))
            }
        }

        impl $op<Decimal> for $int {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: Decimal) -> Self::Output {
                Decimal::from(self).$method(other)
            }
        }

        impl $op<&'_ Decimal> for $int {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: &'_ Decimal) -> Self::Output {
                Decimal::from(self).$method(other)
            }
        }
    };
    ($op: ident { $method: ident } $($int: ty), * $(,)?) => {
        $(impl_arith_with_num!($op { $method } $int);)*
    };
}

macro_rules! impl_arith_try_with_num {
    ($op: ident { $method: ident } $int: ty) => {
        impl $op<$int> for Decimal {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: $int) -> Self::Output {
                self.$method(Decimal::try_from(other).unwrap())
            }
        }

        impl $op<$int> for &'_ Decimal {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: $int) -> Self::Output {
                self.$method(Decimal::try_from(other).unwrap())
            }
        }

        impl $op<Decimal> for $int {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: Decimal) -> Self::Output {
                Decimal::try_from(self).unwrap().$method(other)
            }
        }

        impl $op<&'_ Decimal> for $int {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: &'_ Decimal) -> Self::Output {
                Decimal::try_from(self).unwrap().$method(other)
            }
        }
    };
    ($op: ident { $method: ident } $($int: ty), * $(,)?) => {
        $(impl_arith_try_with_num!($op { $method } $int);)*
    };
}

macro_rules! impl_arith {
    ($op: ident { $method: ident }) => {
        impl $op for Decimal {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: Self) -> Self::Output {
                (&self).$method(other)
            }
        }

        impl $op<&'_ Decimal> for Decimal {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: &Decimal) -> Self::Output {
                (&self).$method(*other)
            }
        }

        impl $op<&'_ Decimal> for &'_ Decimal {
            type Output = Decimal;

            #[inline(always)]
            fn $method(self, other: &Decimal) -> Self::Output {
                self.$method(*other)
            }
        }

        impl_arith_with_num!($op { $method } u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
        impl_arith_try_with_num!($op { $method } f32, f64, i128, u128);
    };
}

impl_arith!(Add { add });
impl_arith!(Sub { sub });
impl_arith!(Mul { mul });
impl_arith!(Div { div });
impl_arith!(Rem { rem });

macro_rules! impl_arith_assign_with_num {
    ($op: ident { $method: ident } $int: ty) => {
        impl $op<$int> for Decimal {
            #[inline(always)]
            fn $method(&mut self, other: $int) {
                self.$method(Decimal::from(other))
            }
        }

        impl $op<$int> for &mut Decimal {
            #[inline(always)]
            fn $method(&mut self, other: $int) {
                (*self).$method(Decimal::from(other))
            }
        }
    };
    ($op: ident { $method: ident } $($int: ty), * $(,)?) => {
        $(impl_arith_assign_with_num!($op { $method } $int);)*
    };
}

macro_rules! impl_arith_assign_try_with_num {
    ($op: ident { $method: ident } $int: ty) => {
        impl $op<$int> for Decimal {
            #[inline(always)]
            fn $method(&mut self, other: $int) {
                self.$method(Decimal::try_from(other).unwrap())
            }
        }

        impl $op<$int> for &mut Decimal {
            #[inline(always)]
            fn $method(&mut self, other: $int) {
                (*self).$method(Decimal::try_from(other).unwrap())
            }
        }
    };
    ($op: ident { $method: ident } $($int: ty), * $(,)?) => {
        $(impl_arith_assign_try_with_num!($op { $method } $int);)*
    };
}

macro_rules! impl_arith_assign {
    ($op: ident { $method: ident }) => {
        impl $op<Decimal> for &mut Decimal {
            #[inline(always)]
            fn $method(&mut self, other: Decimal) {
                (*self).$method(other)
            }
        }

        impl $op<&Decimal> for Decimal {
            #[inline(always)]
            fn $method(&mut self, other: &Decimal) {
                self.$method(*other)
            }
        }

        impl $op<&Decimal> for &mut Decimal {
            #[inline(always)]
            fn $method(&mut self, other: &Decimal) {
                (*self).$method(*other)
            }
        }

        impl_arith_assign_with_num!($op { $method } u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
        impl_arith_assign_try_with_num!($op { $method } f32, f64, i128, u128);
    };
}

impl_arith_assign!(AddAssign { add_assign });
impl_arith_assign!(SubAssign { sub_assign });
impl_arith_assign!(MulAssign { mul_assign });
impl_arith_assign!(DivAssign { div_assign });
impl_arith_assign!(RemAssign { rem_assign });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neg() {
        fn assert_neg(val: &str, expected: &str) {
            let val = val.parse::<Decimal>().unwrap();
            let expected = expected.parse::<Decimal>().unwrap();
            {
                let neg_val = -val;
                assert_eq!(neg_val, expected);
            }
            {
                let neg_val = -(&val);
                assert_eq!(neg_val, expected);
            }
        }

        assert_neg("00000.00000", "0");
        assert_neg("1.0", "-1");
        assert_neg("-1.0", "1");
        assert_neg("1.234", "-1.234");
        assert_neg("-1.234", "1.234");
    }

    #[test]
    fn test_add() {
        fn assert_add(val1: &str, val2: &str, expected: &str) {
            let var1 = val1.parse::<Decimal>().unwrap();
            let var2 = val2.parse::<Decimal>().unwrap();
            let expected = expected.parse::<Decimal>().unwrap();

            let result = var1 + var2;
            assert_eq!(result, expected);
        }

        assert_add("0.000000001", "100000000", "100000000.000000001");
        assert_add("123456789.987654321", "-123456789.987654321", "0");
        assert_add("987654321.123456789", "-987654321.123456789", "0");
        assert_add(
            "123456789.987654321",
            "987654321.123456789",
            "1111111111.11111111",
        );
        assert_add("123456789.987654321", "00000.00000", "123456789.987654321");
        assert_add(
            "123456789.987654321",
            "-987654321.123456789",
            "-864197531.135802468",
        );
        assert_add("00000.00000", "987654321.123456789", "987654321.123456789");
        assert_add("00000.00000", "00000.00000", "0");
        assert_add(
            "00000.00000",
            "-987654321.123456789",
            "-987654321.123456789",
        );
        assert_add(
            "-123456789.987654321",
            "987654321.123456789",
            "864197531.135802468",
        );
        assert_add(
            "-123456789.987654321",
            "00000.00000",
            "-123456789.987654321",
        );
        assert_add(
            "-123456789.987654321",
            "-987654321.123456789",
            "-1111111111.11111111",
        );
        assert_add("-1e28", "-1e122", "-1e122");
    }

    #[test]
    fn test_sub() {
        fn assert_sub(val1: &str, val2: &str, expected1: &str, expected2: &str) {
            let var1 = val1.parse::<Decimal>().unwrap();
            let var2 = val2.parse::<Decimal>().unwrap();
            let expected1 = expected1.parse::<Decimal>().unwrap();
            let expected2 = expected2.parse::<Decimal>().unwrap();

            let result1 = var1 - var2;
            assert_eq!(result1, expected1);

            let result2 = var2 - var1;
            assert_eq!(result2, expected2);
        }

        assert_sub(
            "0.000000001",
            "100000000",
            "-99999999.999999999",
            "99999999.999999999",
        );
        assert_sub(
            "123456789.987654321",
            "123456789.987654321",
            "0.000000000",
            "0.000000000",
        );
        assert_sub(
            "987654321.123456789",
            "987654321.123456789",
            "0.000000000",
            "0.000000000",
        );
        assert_sub(
            "123456789.987654321",
            "987654321.123456789",
            "-864197531.135802468",
            "864197531.135802468",
        );
        assert_sub(
            "123456789.987654321",
            "00000.00000",
            "123456789.987654321",
            "-123456789.987654321",
        );
        assert_sub(
            "123456789.987654321",
            "-987654321.123456789",
            "1111111111.111111110",
            "-1111111111.111111110",
        );
        assert_sub(
            "00000.00000",
            "987654321.123456789",
            "-987654321.123456789",
            "987654321.123456789",
        );
        assert_sub("00000.00000", "00000.00000", "0.00000", "0.00000");
        assert_sub(
            "00000.00000",
            "-987654321.123456789",
            "987654321.123456789",
            "-987654321.123456789",
        );
        assert_sub(
            "-123456789.987654321",
            "987654321.123456789",
            "-1111111111.111111110",
            "1111111111.111111110",
        );
        assert_sub(
            "-123456789.987654321",
            "00000.00000",
            "-123456789.987654321",
            "123456789.987654321",
        );
        assert_sub(
            "-123456789.987654321",
            "-987654321.123456789",
            "864197531.135802468",
            "-864197531.135802468",
        );
        assert_sub("-1e28", "-1e122", "1e122", "-1e122");
    }

    #[test]
    fn test_mul() {
        fn assert_mul(val1: &str, val2: &str, expected: &str) {
            let var1 = val1.parse::<Decimal>().unwrap();
            let var2 = val2.parse::<Decimal>().unwrap();
            let expected = expected.parse::<Decimal>().unwrap();

            let result = var1 * var2;
            assert_eq!(result, expected);
        }

        assert_mul("0.000000001", "100000000", "0.1");
        assert_mul(
            "123456789.987654321",
            "-123456789.987654321",
            "-15241578994055784.200731595789971041",
        );
        assert_mul(
            "987654321.123456789",
            "-987654321.123456789",
            "-975461058033836303.240512116750190521",
        );
        assert_mul(
            "123456789.987654321",
            "987654321.123456789",
            "121932632103337905.662094193112635269",
        );
        assert_mul("123456789.987654321", "00000.00000", "0");
        assert_mul(
            "123456789.987654321",
            "-987654321.123456789",
            "-121932632103337905.662094193112635269",
        );
        assert_mul("00000.00000", "987654321.123456789", "0");
        assert_mul("00000.00000", "00000.00000", "0");
        assert_mul("00000.00000", "-987654321.123456789", "0");
        assert_mul(
            "-123456789.987654321",
            "987654321.123456789",
            "-121932632103337905.662094193112635269",
        );
        assert_mul("-123456789.987654321", "00000.00000", "0");
        assert_mul(
            "-123456789.987654321",
            "-987654321.123456789",
            "121932632103337905.662094193112635269",
        );
    }

    #[test]
    fn test_div() {
        fn assert_div(val1: &str, val2: &str, expected: &str) {
            let var1 = val1.parse::<Decimal>().unwrap();
            let var2 = val2.parse::<Decimal>().unwrap();
            let expected = expected.parse::<Decimal>().unwrap();

            let result = var1 / var2;
            assert_eq!(result, expected);
        }

        assert_div("0.000000001", "100000000", "0.00000000000000001");
        assert_div("100000000", "0.000000001", "100000000000000000");
        assert_div("123456789.987654321", "123456789.987654321", "1");
        assert_div("987654321.123456789", "987654321.123456789", "1");
        assert_div(
            "123456789.987654321",
            "987654321.123456789",
            "0.12499999984531250017595703104984887718",
        );
        assert_div(
            "987654321.123456789",
            "123456789.987654321",
            "8.000000009900000000990000000099",
        );
        assert_div("00000.00000", "123456789.987654321", "0");
        assert_div(
            "123456789.987654321",
            "-987654321.123456789",
            "-0.12499999984531250017595703104984887718",
        );
        assert_div(
            "-987654321.123456789",
            "123456789.987654321",
            "-8.000000009900000000990000000099",
        );
        assert_div("00000.00000", "987654321.123456789", "0");
        assert_div("00000.00000", "-987654321.123456789", "0");
        assert_div(
            "-123456789.987654321",
            "987654321.123456789",
            "-0.12499999984531250017595703104984887718",
        );
        assert_div(
            "987654321.123456789",
            "-123456789.987654321",
            "-8.000000009900000000990000000099",
        );
        assert_div("00000.00000", "-123456789.987654321", "0");
        assert_div(
            "-123456789.987654321",
            "-987654321.123456789",
            "0.12499999984531250017595703104984887718",
        );
        assert_div(
            "-987654321.123456789",
            "-123456789.987654321",
            "8.000000009900000000990000000099",
        );
        assert_div("1", "3", "0.33333333333333333333333333333333333333");
        assert_div("1", "33", "0.030303030303030303030303030303030303030");
        assert_div(
            "-3.1415926",
            "-0.12345678901234567890123456789012345678",
            "25.446900289022102624101133879320318304",
        );
    }

    #[test]
    fn test_rem() {
        fn assert_rem(val1: &str, val2: &str, expected: &str) {
            let var1 = val1.parse::<Decimal>().unwrap();
            let var2 = val2.parse::<Decimal>().unwrap();
            let expected = expected.parse::<Decimal>().unwrap();

            let result = var1 % var2;
            assert_eq!(result, expected);
        }

        assert_rem("0.000000001", "100000000", "0.000000001");
        assert_rem("100000000", "0.000000001", "0.000000000");
        assert_rem("123456789.987654321", "123456789.987654321", "0");
        assert_rem("987654321.123456789", "987654321.123456789", "0");
        assert_rem(
            "123456789.987654321",
            "987654321.123456789",
            "123456789.987654321",
        );
        assert_rem("987654321.123456789", "123456789.987654321", "1.222222221");
        assert_rem("00000.00000", "123456789.987654321", "0");
        assert_rem(
            "123456789.987654321",
            "-987654321.123456789",
            "123456789.987654321",
        );
        assert_rem(
            "-987654321.123456789",
            "123456789.987654321",
            "-1.222222221",
        );
        assert_rem("00000.00000", "987654321.123456789", "0.000000000");
        assert_rem("00000.00000", "-987654321.123456789", "0.000000000");
        assert_rem(
            "-123456789.987654321",
            "987654321.123456789",
            "-123456789.987654321",
        );
        assert_rem("987654321.123456789", "-123456789.987654321", "1.222222221");
        assert_rem("00000.00000", "-123456789.987654321", "0.000000000");
        assert_rem(
            "-123456789.987654321",
            "-987654321.123456789",
            "-123456789.987654321",
        );
        assert_rem(
            "-987654321.123456789",
            "-123456789.987654321",
            "-1.222222221",
        );
        assert_rem("100", "5", "0");
        assert_rem("2e1", "1", "0");
        assert_rem("2", "1", "0");
        assert_rem("1", "3", "1");
        assert_rem("1", "0.5", "0");
        assert_rem("1.5", "1", "0.5");
        assert_rem("1", "3e-2", "1e-2");
        assert_rem("10", "0.003", "0.001");
        assert_rem("3", "2", "1");
        assert_rem("-3", "2", "-1");
        assert_rem("3", "-2", "1");
        assert_rem("-3", "-2", "-1");
        assert_rem("-3", "-1", "0");
        assert_rem("12.34", "1.233", "0.01");
        assert_rem("5e42", "0.3", "0.2");
        assert_rem("-5e42", "0.3", "-0.2");
        assert_rem("5e42", "-0.3", "0.2");
        assert_rem("-5e42", "-0.3", "-0.2");
        assert_rem("5e42", "0.03", "0.02");
        assert_rem("5e42", "3", "2");
        assert_rem("5e60", "3", "2");
        assert_rem("5e60", "300", "200");
        assert_rem("5e76", "3", "2");
        assert_rem("5e77", "3", "2e39");
        assert_rem("5e-42", "3e-84", "2e-84");
        assert_rem("5e126", "3e-130", "2e88");
        assert_rem("4e126", "3e-130", "1e88");
        assert_rem(
            "99999999999999999999999999999999999999e126",
            "7e-130",
            "2e88",
        );
        assert_rem("3", "5e42", "3");
    }

    #[test]
    fn test_sum() {
        fn assert_sum(vals: &[&str], expected: &str) {
            let result: Decimal = vals.iter().map(|val| val.parse::<Decimal>().unwrap()).sum();
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(result, expected);
        }

        assert_sum(&["1", "10", "100", "1000", "10000"], "11111");
        assert_sum(&["-1", "-10", "-100", "-1000", "-10000"], "-11111");
        assert_sum(&["0", "0", "0", "0", "0"], "0");
    }

    #[test]
    fn test_product() {
        fn assert_product(vals: &[&str], expected: &str) {
            let result: Decimal = vals
                .iter()
                .map(|val| val.parse::<Decimal>().unwrap())
                .product();
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(result, expected);
        }

        assert_product(&["1", "2", "3", "4", "5"], "120");
        assert_product(&["-1", "-2", "-3", "-4", "-5"], "-120");
        assert_product(&["0", "0", "0", "0", "0"], "0");
    }
}

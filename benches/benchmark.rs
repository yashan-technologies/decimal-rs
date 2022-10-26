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

//! decimal-rs benchmark

use bencher::{benchmark_group, benchmark_main, black_box, Bencher};
use decimal_rs::{Decimal, DecimalConvertError, DECIMAL128, MAX_BINARY_SIZE};
use std::collections::hash_map::DefaultHasher;
use std::convert::{TryFrom, TryInto};
use std::hash::Hash;

#[inline(always)]
fn parse(s: &str) -> Decimal {
    s.parse().unwrap()
}

fn decimal_parse(bench: &mut Bencher) {
    bench.iter(|| {
        let _n = parse(black_box("12345678901.23456789"));
    })
}

fn decimal_to_string(bench: &mut Bencher) {
    let val = parse("12345678901.23456789");
    bench.iter(|| {
        let _n = black_box(&val).to_string();
    })
}

fn decimal_precision(bench: &mut Bencher) {
    let val = parse("12345678901.23456789");
    bench.iter(|| {
        let _n = black_box(&val).precision();
        black_box(_n);
    })
}

#[inline(always)]
fn try_from<T: TryInto<Decimal, Error = DecimalConvertError>>(val: T) -> Decimal {
    val.try_into().unwrap()
}

#[allow(clippy::excessive_precision)]
fn decimal_from_f64(bench: &mut Bencher) {
    bench.iter(|| {
        let _n = try_from(black_box(12345678901.23456789_f64));
    })
}

fn decimal_into_f64(bench: &mut Bencher) {
    let val = parse("12345678901.23456789");
    bench.iter(|| {
        black_box(f64::from(black_box(&val)));
    })
}

fn decimal_into_u64(bench: &mut Bencher) {
    let val = parse("12345678901.23456789");
    bench.iter(|| {
        let _n = u64::try_from(black_box(&val)).unwrap();
    })
}

#[inline(always)]
fn add(x: &Decimal, y: &Decimal) -> Decimal {
    x + y
}

fn decimal_add(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    let y = parse("123456.7890123456789");
    bench.iter(|| {
        let _n = add(black_box(&x), black_box(&y));
    })
}

#[inline(always)]
fn sub(x: &Decimal, y: &Decimal) -> Decimal {
    x - y
}

fn decimal_sub(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    let y = parse("123456.7890123456789");
    bench.iter(|| {
        let _n = sub(black_box(&x), black_box(&y));
    })
}

#[inline(always)]
fn mul(x: &Decimal, y: &Decimal) -> Decimal {
    x * y
}

fn decimal_mul(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    let y = parse("123456.7890123456789");
    bench.iter(|| {
        let _n = mul(black_box(&x), black_box(&y));
    })
}

#[inline(always)]
fn div(x: &Decimal, y: &Decimal) -> Decimal {
    x / y
}

fn decimal_div(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    let y = parse("123456.7890123456789");
    bench.iter(|| {
        let _n = div(black_box(&x), black_box(&y));
    })
}

fn decimal_rem(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    let y = parse("123456.7890123456789");
    bench.iter(|| {
        let _n = black_box(&x) % black_box(&y);
    })
}

fn decimal_encode(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    let mut buf = [0; MAX_BINARY_SIZE];
    bench.iter(|| {
        let _n = black_box(black_box(&x).encode(&mut buf[..]).unwrap());
    })
}

fn decimal_decode(bench: &mut Bencher) {
    let mut buf = Vec::new();
    parse("12345678901.23456789").encode(&mut buf).unwrap();
    bench.iter(|| {
        let _n = black_box(Decimal::decode(black_box(&buf)));
    })
}

fn decimal_normalize(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    bench.iter(|| {
        let _n = black_box(black_box(&x).normalize());
    })
}

fn decimal_hash(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    let mut hasher = DefaultHasher::new();
    bench.iter(|| {
        black_box(&x).hash(&mut hasher);
    })
}

fn decimal_cmp(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    let y = parse("12345.67890123456789");
    bench.iter(|| {
        let _n = black_box(x > y);
    })
}

fn decimal_sqrt(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    bench.iter(|| {
        let _n = black_box(&x).sqrt();
    })
}

fn decimal_sci_zero(bench: &mut Bencher) {
    let x = parse("0.0");
    let mut s = String::with_capacity(100);
    bench.iter(|| {
        s.clear();
        // "0"
        let _n = black_box(&x).format_with_sci(1, &mut s);
    })
}

fn decimal_sci_normal(bench: &mut Bencher) {
    let x = parse("1000");
    let mut s = String::with_capacity(100);
    bench.iter(|| {
        s.clear();
        // "1000"
        let _n = black_box(&x).format_with_sci(4, &mut s);
    })
}

fn decimal_sci_normal_round(bench: &mut Bencher) {
    let x = parse(".0000123456789");
    let mut s = String::with_capacity(100);
    bench.iter(|| {
        s.clear();
        // ".000012346"
        let _n = black_box(&x).format_with_sci(10, &mut s);
    })
}

fn decimal_sci_int(bench: &mut Bencher) {
    let x = parse("1234567890.123456789");
    let mut s = String::with_capacity(100);
    bench.iter(|| {
        s.clear();
        // "1.2E+09"
        let _n = black_box(&x).format_with_sci(7, &mut s);
    })
}

fn decimal_sci_fraction(bench: &mut Bencher) {
    let x = parse(".00000000123456789");
    let mut s = String::with_capacity(100);
    bench.iter(|| {
        s.clear();
        // "1.2E-09"
        let _n = black_box(&x).format_with_sci(7, &mut s);
    })
}

fn decimal_sci_supply_zero(bench: &mut Bencher) {
    let x = parse("0.1E-126");
    let mut s = String::with_capacity(100);
    bench.iter(|| {
        s.clear();
        let _n = black_box(&x).format_with_sci(127, &mut s);
    })
}

fn decimal_hex(bench: &mut Bencher) {
    let x = parse("3534.33");
    let mut s = String::with_capacity(64);
    bench.iter(|| {
        s.clear();
        let _n = black_box(&x).format_to_hex(true, &mut s);
    })
}

fn decimal_pow(bench: &mut Bencher) {
    let x = parse("12.3456");
    let y = parse("50.123456");
    bench.iter(|| {
        let _n = black_box(&x).checked_pow(&y);
    })
}

fn decimal_ln(bench: &mut Bencher) {
    let x = parse("1234567890123456.789123456");
    bench.iter(|| {
        let _n = black_box(&x).ln();
    })
}

fn decimal_exp(bench: &mut Bencher) {
    let x = parse("259.123456789");
    bench.iter(|| {
        let _n = black_box(&x).exp();
    })
}

fn decimal_ceil_100_times(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    bench.iter(|| {
        for _ in 0..100 {
            let _n = black_box(&x).ceil();
        }
    })
}

fn decimal_floor_100_times(bench: &mut Bencher) {
    let x = parse("12345678901.23456789");
    bench.iter(|| {
        for _ in 0..100 {
            let _n = black_box(&x).floor();
        }
    })
}

#[inline(always)]
fn add_with_same_scale(x: &Decimal, y: &Decimal) -> Decimal {
    unsafe { x.add_with_same_scale_unchecked::<DECIMAL128>(y, 8) }
}

fn decimal_uncheck_add_same_scale_100_times(bench: &mut Bencher) {
    let x = parse("901.23456789");
    let y = parse("8901.23456789");
    bench.iter(|| {
        for _ in 0..100 {
            let _n = add_with_same_scale(black_box(&x), black_box(&y));
        }
    })
}

#[inline(always)]
fn add_with_same_scale_negative(x: &Decimal, y: &Decimal) -> Decimal {
    unsafe { x.add_with_same_scale_and_negative_unchecked::<DECIMAL128>(y, 8, true) }
}

fn decimal_uncheck_add_same_scale_negative_100_times(bench: &mut Bencher) {
    let x = parse("1891.23456789");
    let y = parse("6701.23456789");
    bench.iter(|| {
        for _ in 0..100 {
            let _n = add_with_same_scale_negative(black_box(&x), black_box(&y));
        }
    })
}

#[inline(always)]
fn sub_with_same_scale(x: &Decimal, y: &Decimal) -> Decimal {
    unsafe { x.sub_with_same_scale_unchecked::<DECIMAL128>(y, 8) }
}

fn decimal_uncheck_sub_100_times(bench: &mut Bencher) {
    let x = parse("11.23456789");
    let y = parse("71.23456789");
    bench.iter(|| {
        for _ in 0..100 {
            let _n = sub_with_same_scale(black_box(&x), black_box(&y));
        }
    })
}

#[inline(always)]
fn mull_unchecked(x: &Decimal, y: &Decimal) -> Decimal {
    unsafe { x.mul_unchecked::<DECIMAL128>(y, 16) }
}

fn decimal_uncheck_mul_100_times(bench: &mut Bencher) {
    let x = parse("1901.23456789");
    let y = parse("7901.23456789");
    bench.iter(|| {
        for _ in 0..100 {
            let _n = mull_unchecked(black_box(&x), black_box(&y));
        }
    })
}

#[inline(always)]
fn cmp_zero(x: i128) -> bool {
    x != 0
}

fn i128_cmp_zero_100_times(bench: &mut Bencher) {
    let x = 12345678901;
    bench.iter(|| {
        for _ in 0..100 {
            let _n = cmp_zero(black_box(x));
        }
    })
}

benchmark_group!(
    decimal_benches,
    decimal_parse,
    decimal_to_string,
    decimal_precision,
    decimal_into_f64,
    decimal_from_f64,
    decimal_into_u64,
    decimal_add,
    decimal_sub,
    decimal_mul,
    decimal_div,
    decimal_rem,
    decimal_encode,
    decimal_decode,
    decimal_normalize,
    decimal_hash,
    decimal_cmp,
    decimal_sqrt,
    decimal_sci_zero,
    decimal_sci_normal,
    decimal_sci_normal_round,
    decimal_sci_int,
    decimal_sci_fraction,
    decimal_sci_supply_zero,
    decimal_hex,
    decimal_pow,
    decimal_ln,
    decimal_exp,
    decimal_ceil_100_times,
    decimal_floor_100_times,
    decimal_uncheck_add_same_scale_100_times,
    decimal_uncheck_add_same_scale_negative_100_times,
    decimal_uncheck_sub_100_times,
    decimal_uncheck_mul_100_times,
    i128_cmp_zero_100_times
);

benchmark_main!(decimal_benches);

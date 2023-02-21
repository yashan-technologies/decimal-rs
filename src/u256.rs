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

//! Unsigned 256-bit integer.

use crate::decimal::MAX_PRECISION;
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Rem, Shl, Shr, Sub};

pub static POWERS_10: [U256; (MAX_PRECISION * 2 + 1) as usize] = [
    U256::from_u128(1, 0),
    U256::from_u128(10, 0),
    U256::from_u128(100, 0),
    U256::from_u128(1000, 0),
    U256::from_u128(10000, 0),
    U256::from_u128(100000, 0),
    U256::from_u128(1000000, 0),
    U256::from_u128(10000000, 0),
    U256::from_u128(100000000, 0),
    U256::from_u128(1000000000, 0),
    U256::from_u128(10000000000, 0),
    U256::from_u128(100000000000, 0),
    U256::from_u128(1000000000000, 0),
    U256::from_u128(10000000000000, 0),
    U256::from_u128(100000000000000, 0),
    U256::from_u128(1000000000000000, 0),
    U256::from_u128(10000000000000000, 0),
    U256::from_u128(100000000000000000, 0),
    U256::from_u128(1000000000000000000, 0),
    U256::from_u128(10000000000000000000, 0),
    U256::from_u128(100000000000000000000, 0),
    U256::from_u128(1000000000000000000000, 0),
    U256::from_u128(10000000000000000000000, 0),
    U256::from_u128(100000000000000000000000, 0),
    U256::from_u128(1000000000000000000000000, 0),
    U256::from_u128(10000000000000000000000000, 0),
    U256::from_u128(100000000000000000000000000, 0),
    U256::from_u128(1000000000000000000000000000, 0),
    U256::from_u128(10000000000000000000000000000, 0),
    U256::from_u128(100000000000000000000000000000, 0),
    U256::from_u128(1000000000000000000000000000000, 0),
    U256::from_u128(10000000000000000000000000000000, 0),
    U256::from_u128(100000000000000000000000000000000, 0),
    U256::from_u128(1000000000000000000000000000000000, 0),
    U256::from_u128(10000000000000000000000000000000000, 0),
    U256::from_u128(100000000000000000000000000000000000, 0),
    U256::from_u128(1000000000000000000000000000000000000, 0),
    U256::from_u128(10000000000000000000000000000000000000, 0),
    U256::from_u128(100000000000000000000000000000000000000, 0),
    U256::from_u128(319435266158123073073250785136463577088, 2),
    U256::from_u128(131811359292784559562136384478721867776, 29),
    U256::from_u128(297266492165030205231240022491914043392, 293),
    U256::from_u128(250405986282794344605403365464994742272, 2938),
    U256::from_u128(122083294381374201810411402627569942528, 29387),
    U256::from_u128(199985843050926627713990203980394790912, 293873),
    U256::from_u128(298446595904573959823029002645106851840, 2938735),
    U256::from_u128(262207023678231890523293166996922826752, 29387358),
    U256::from_u128(240093668335749660989309417946850787328, 293873587),
    U256::from_u128(18960114910927365649471927446130393088, 2938735877),
    U256::from_u128(189601149109273656494719274461303930880, 29387358770),
    U256::from_u128(194599656488044247630319707454198251520, 293873587705),
    U256::from_u128(244584730275750158986324037383141457920, 2938735877055),
    U256::from_u128(63870734310932345619618121809037099008, 29387358770557),
    U256::from_u128(298424976188384992732806610658602778624, 293873587705571),
    U256::from_u128(261990826516342219621069247131882094592, 2938735877055718),
    U256::from_u128(237931696716852951967070219296443465728, 29387358770557187),
    U256::from_u128(337622765642898738890454548373825388544, 293873587705571876),
    U256::from_u128(313686354140541217734174016852339982336, 2938735877055718769),
    U256::from_u128(74322239116966006171368701637485920256, 29387358770557187699),
    U256::from_u128(62657657327783134786937801511322779648, 293873587705571876992),
    U256::from_u128(286294206356892884406003407681459585024, 2938735877055718769921),
    U256::from_u128(140683128201421136353037217360450158592, 29387358770557187699218),
    U256::from_u128(45701814330457509676873743877428740096, 293873587705571876992184),
    U256::from_u128(116735776383636633305362831342519189504, 2938735877055718769921841),
    U256::from_u128(146510663073550942663504491129887260672, 29387358770557187699218413),
    U256::from_u128(103977163051755572781546481571799760896, 293873587705571876992184134),
    U256::from_u128(18924529754740337425340993422692974592, 2938735877055718769921841343),
    U256::from_u128(189245297547403374253409934226929745920, 29387358770557187699218413430),
    U256::from_u128(191041140869341425217226305110456401920, 293873587705571876992184134305),
    U256::from_u128(208999574088721934855390013945722961920, 2938735877055718769921841343055),
    U256::from_u128(48301539361588567773652494866620350464, 29387358770557187699218413430556),
    U256::from_u128(
        142733026694947214273150341234435293184,
        293873587705571876992184134305561,
    ),
    U256::from_u128(
        66200799265718288878004982617280086016,
        2938735877055718769921841343055614,
    ),
    U256::from_u128(
        321725625736244425316675218741032648704,
        29387358770557187699218413430556141,
    ),
    U256::from_u128(
        154714955073998081996380720524412583936,
        293873587705571876992184134305561419,
    ),
    U256::from_u128(
        186020083056226966110308775517052993536,
        2938735877055718769921841343055614194,
    ),
    U256::from_u128(
        158788995957577343786214718011688878080,
        29387358770557187699218413430556141945,
    ),
];

pub static ROUNDINGS: [U256; (MAX_PRECISION * 2 + 1) as usize] = [
    U256::from_u128(0, 0),
    U256::from_u128(5, 0),
    U256::from_u128(50, 0),
    U256::from_u128(500, 0),
    U256::from_u128(5000, 0),
    U256::from_u128(50000, 0),
    U256::from_u128(500000, 0),
    U256::from_u128(5000000, 0),
    U256::from_u128(50000000, 0),
    U256::from_u128(500000000, 0),
    U256::from_u128(5000000000, 0),
    U256::from_u128(50000000000, 0),
    U256::from_u128(500000000000, 0),
    U256::from_u128(5000000000000, 0),
    U256::from_u128(50000000000000, 0),
    U256::from_u128(500000000000000, 0),
    U256::from_u128(5000000000000000, 0),
    U256::from_u128(50000000000000000, 0),
    U256::from_u128(500000000000000000, 0),
    U256::from_u128(5000000000000000000, 0),
    U256::from_u128(50000000000000000000, 0),
    U256::from_u128(500000000000000000000, 0),
    U256::from_u128(5000000000000000000000, 0),
    U256::from_u128(50000000000000000000000, 0),
    U256::from_u128(500000000000000000000000, 0),
    U256::from_u128(5000000000000000000000000, 0),
    U256::from_u128(50000000000000000000000000, 0),
    U256::from_u128(500000000000000000000000000, 0),
    U256::from_u128(5000000000000000000000000000, 0),
    U256::from_u128(50000000000000000000000000000, 0),
    U256::from_u128(500000000000000000000000000000, 0),
    U256::from_u128(5000000000000000000000000000000, 0),
    U256::from_u128(50000000000000000000000000000000, 0),
    U256::from_u128(500000000000000000000000000000000, 0),
    U256::from_u128(5000000000000000000000000000000000, 0),
    U256::from_u128(50000000000000000000000000000000000, 0),
    U256::from_u128(500000000000000000000000000000000000, 0),
    U256::from_u128(5000000000000000000000000000000000000, 0),
    U256::from_u128(50000000000000000000000000000000000000, 0),
    U256::from_u128(159717633079061536536625392568231788544, 1),
    U256::from_u128(236046863106861511512755495955245039616, 14),
    U256::from_u128(318774429542984334347307314961841127424, 146),
    U256::from_u128(125202993141397172302701682732497371136, 1469),
    U256::from_u128(231182830651156332636893005029669076992, 14693),
    U256::from_u128(270134104985932545588682405706081501184, 146936),
    U256::from_u128(319364481412756211643201805038437531648, 1469367),
    U256::from_u128(131103511839115945261646583498461413376, 14693679),
    U256::from_u128(290188017628344062226342012689309499392, 146936793),
    U256::from_u128(179621240915932914556423267438949302272, 1469367938),
    U256::from_u128(94800574554636828247359637230651965440, 14693679385),
    U256::from_u128(267441011704491355546847157442983231488, 146936793852),
    U256::from_u128(292433548598344311224849322407454834688, 1469367938527),
    U256::from_u128(202076550615935404541496364620402655232, 14693679385278),
    U256::from_u128(319353671554661728098090609045185495040, 146936793852785),
    U256::from_u128(130995413258171109810534623565941047296, 1469367938527859),
    U256::from_u128(289107031818895707715222413364105838592, 14693679385278593),
    U256::from_u128(168811382821449369445227274186912694272, 146936793852785938),
    U256::from_u128(326984360530739840598774312142054096896, 1469367938527859384),
    U256::from_u128(207302303018952234817371654534627065856, 14693679385278593849),
    U256::from_u128(31328828663891567393468900755661389824, 146936793852785938496),
    U256::from_u128(313288286638915673934689007556613898240, 1469367938527859384960),
    U256::from_u128(70341564100710568176518608680225079296, 14693679385278593849609),
    U256::from_u128(22850907165228754838436871938714370048, 146936793852785938496092),
    U256::from_u128(228509071652287548384368719387143700480, 1469367938527859384960920),
    U256::from_u128(243396514997244703063439549280827736064, 14693679385278593849609206),
    U256::from_u128(51988581525877786390773240785899880448, 146936793852785938496092067),
    U256::from_u128(179603448337839400444357800427230593024, 1469367938527859384960920671),
    U256::from_u128(94622648773701687126704967113464872960, 14693679385278593849609206715),
    U256::from_u128(265661753895139944340300456271112306688, 146936793852785938496092067152),
    U256::from_u128(274640970504830199159382310688745586688, 1469367938527859384960920671527),
    U256::from_u128(24150769680794283886826247433310175232, 14693679385278593849609206715278),
    U256::from_u128(
        241507696807942838868262474333101752320,
        146936793852785938496092067152780,
    ),
    U256::from_u128(
        33100399632859144439002491308640043008,
        1469367938527859384960920671527807,
    ),
    U256::from_u128(
        331003996328591444390024913086400430080,
        14693679385278593849609206715278070,
    ),
    U256::from_u128(
        247498660997468272729877663978090397696,
        146936793852785938496092067152780709,
    ),
    U256::from_u128(
        93010041528113483055154387758526496768,
        1469367938527859384960920671527807097,
    ),
    U256::from_u128(
        249535681439257903624794662721728544768,
        14693679385278593849609206715278070972,
    ),
];

const N_UDWORD_BITS: u32 = 128;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct U256 {
    high: u128,
    low: u128,
}

impl U256 {
    pub const ZERO: U256 = U256::from_u128(0, 0);
    pub const ONE: U256 = U256::from_u128(1, 0);

    #[inline(always)]
    pub const fn from_u128(low: u128, high: u128) -> U256 {
        U256 { high, low }
    }

    #[inline(always)]
    pub fn low(&self) -> u128 {
        self.low
    }

    #[inline(always)]
    pub fn high(&self) -> u128 {
        self.high
    }

    #[inline]
    pub fn count_digits(&self) -> u32 {
        match POWERS_10.binary_search(self) {
            Ok(pos) => pos as u32 + 1,
            Err(pos) => {
                if pos == 0 {
                    pos as u32 + 1
                } else {
                    pos as u32
                }
            }
        }
    }

    #[inline]
    pub fn is_decimal_overflowed(&self) -> bool {
        if self.high() > 0 {
            true
        } else {
            self.low() >= POWERS_10[MAX_PRECISION as usize].low()
        }
    }

    #[inline(always)]
    pub fn checked_add<T: Into<U256>>(&self, other: T) -> Option<U256> {
        let (res, overflow) = self.overflowing_add(other.into());
        if overflow {
            None
        } else {
            Some(res)
        }
    }

    #[inline(always)]
    pub fn checked_sub<T: Into<U256>>(&self, other: T) -> Option<U256> {
        let (res, overflow) = self.overflowing_sub(other.into());
        if overflow {
            None
        } else {
            Some(res)
        }
    }

    #[inline(always)]
    pub fn checked_mul<T: Into<U256>>(&self, other: T) -> Option<U256> {
        let (res, overflow) = self.overflowing_mul(other.into());
        if overflow {
            None
        } else {
            Some(res)
        }
    }

    #[inline(always)]
    pub fn wrapping_mul(&self, other: U256) -> U256 {
        let res = U256::mul128(self.low(), other.low());
        let lo_hi = self.low().wrapping_mul(other.high());
        let hi_lo = self.high().wrapping_mul(other.low());
        let high = res.high().wrapping_add(lo_hi).wrapping_add(hi_lo);
        U256::from_u128(res.low(), high)
    }

    #[inline]
    pub fn div_rem<T: Into<U256>>(&self, other: T) -> (U256, U256) {
        let other = other.into();

        if self.high() | other.high() == 0 {
            (
                U256::from(self.low() / other.low()),
                U256::from(self.low() % other.low()),
            )
        } else if &other > self {
            (U256::from(0u128), *self)
        } else if other.high() == 0 {
            let mut remainder = 0;
            let quotient;
            if self.high() < other.low() {
                quotient = U256::from(udiv256_by_128_to_128(
                    self.high(),
                    self.low(),
                    other.low(),
                    &mut remainder,
                ));
                (quotient, U256::from(remainder))
            } else {
                quotient = U256::from_u128(
                    udiv256_by_128_to_128(self.high() % other.low(), self.low(), other.low(), &mut remainder),
                    self.high() / other.low(),
                );
                (quotient, U256::from(remainder))
            }
        } else {
            knuth_div_mod(self, &other)
        }
    }

    #[inline]
    pub fn div128_round(&self, other: u128) -> U256 {
        let (result, rem) = self.div_rem(other);

        if rem == 0 {
            result
        } else {
            // rounding:
            //    remain / other >= 1 / 2
            // => other - remain <= remain
            let sub_result = other - rem;
            if rem >= sub_result {
                result + 1
            } else {
                result
            }
        }
    }

    #[inline]
    pub fn cmp128(&self, other: u128) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }

    #[inline(always)]
    pub fn add128(left: u128, right: u128) -> U256 {
        U256::from(left) + U256::from(right)
    }

    #[inline(always)]
    pub fn mul128(left: u128, right: u128) -> U256 {
        const BITS_IN_DWORD_2: u32 = 64;
        const LOWER_MASK: u128 = u128::MAX >> BITS_IN_DWORD_2;

        let mut low = (left & LOWER_MASK) * (right & LOWER_MASK);
        let mut t = low >> BITS_IN_DWORD_2;
        low &= LOWER_MASK;
        t += (left >> BITS_IN_DWORD_2) * (right & LOWER_MASK);
        let mut high = t >> BITS_IN_DWORD_2;
        t &= LOWER_MASK;
        t += (right >> BITS_IN_DWORD_2) * (left & LOWER_MASK);
        low += (t & LOWER_MASK) << BITS_IN_DWORD_2;
        high += t >> BITS_IN_DWORD_2;
        high += (left >> BITS_IN_DWORD_2) * (right >> BITS_IN_DWORD_2);

        U256::from_u128(low, high)
    }

    #[inline]
    fn overflowing_add(self, other: U256) -> (U256, bool) {
        let (low, carry) = self.low().overflowing_add(other.low());
        let (high, carry_overflow) = self.high().overflowing_add(carry as u128);
        let (high, high_overflow) = high.overflowing_add(other.high());
        (U256::from_u128(low, high), carry_overflow || high_overflow)
    }

    #[inline]
    fn overflowing_sub(self, other: U256) -> (U256, bool) {
        let (low, borrow) = self.low().overflowing_sub(other.low());
        let (high, borrow_overflow) = self.high().overflowing_sub(borrow as _);
        let (high, high_overflow) = high.overflowing_sub(other.high());
        (U256::from_u128(low, high), borrow_overflow || high_overflow)
    }

    #[inline]
    fn overflowing_mul(self, other: U256) -> (U256, bool) {
        let res = U256::mul128(self.low(), other.low());
        let (lo_hi, lo_hi_overflow) = self.low().overflowing_mul(other.high());
        let (hi_lo, hi_lo_overflow) = self.high().overflowing_mul(other.low());
        let (high, add_overflow1) = res.high().overflowing_add(lo_hi);
        let (high, add_overflow2) = high.overflowing_add(hi_lo);
        let high_overflow = self.high() != 0 && other.high() != 0;
        (
            U256::from_u128(res.low(), high),
            lo_hi_overflow || hi_lo_overflow || add_overflow1 || add_overflow2 || high_overflow,
        )
    }
}

#[inline(always)]
fn udiv256_by_128_to_128(u1: u128, u0: u128, mut v: u128, r: &mut u128) -> u128 {
    const B: u128 = 1 << (N_UDWORD_BITS / 2); // Number base (128 bits)
    let (un1, un0): (u128, u128); // Norm. dividend LSD's
    let (vn1, vn0): (u128, u128); // Norm. divisor digits
    let (mut q1, mut q0): (u128, u128); // Quotient digits
    let (un128, un21, un10): (u128, u128, u128); // Dividend digit pairs

    let s = v.leading_zeros();
    if s > 0 {
        // Normalize the divisor.
        v <<= s;
        un128 = (u1 << s) | (u0 >> (N_UDWORD_BITS - s));
        un10 = u0 << s; // Shift dividend left
    } else {
        // Avoid undefined behavior of (u0 >> 64).
        un128 = u1;
        un10 = u0;
    }

    // Break divisor up into two 64-bit digits.
    vn1 = v >> (N_UDWORD_BITS / 2);
    vn0 = v & 0xFFFF_FFFF_FFFF_FFFF;

    // Break right half of dividend into two digits.
    un1 = un10 >> (N_UDWORD_BITS / 2);
    un0 = un10 & 0xFFFF_FFFF_FFFF_FFFF;

    // Compute the first quotient digit, q1.
    q1 = un128 / vn1;
    let mut rhat = un128 - q1 * vn1;

    // q1 has at most error 2. No more than 2 iterations.
    while q1 >= B || q1 * vn0 > B * rhat + un1 {
        q1 -= 1;
        rhat += vn1;
        if rhat >= B {
            break;
        }
    }

    un21 = un128.wrapping_mul(B).wrapping_add(un1).wrapping_sub(q1.wrapping_mul(v));

    // Compute the second quotient digit.
    q0 = un21 / vn1;
    rhat = un21 - q0 * vn1;

    // q0 has at most error 2. No more than 2 iterations.
    while q0 >= B || q0 * vn0 > B * rhat + un0 {
        q0 -= 1;
        rhat += vn1;
        if rhat >= B {
            break;
        }
    }

    *r = (un21.wrapping_mul(B).wrapping_add(un0).wrapping_sub(q0.wrapping_mul(v))) >> s;
    q1 * B + q0
}

#[inline]
fn full_shl(a: &U256, shift: u32) -> [u128; 3] {
    debug_assert!(shift < N_UDWORD_BITS);
    let mut u = [0_u128; 3];
    let u_lo = a.low() << shift;
    let u_hi = *a >> (N_UDWORD_BITS - shift);
    u[0] = u_lo;
    u[1] = u_hi.low();
    u[2] = u_hi.high();

    u
}

#[inline]
fn full_shr(u: &[u128; 3], shift: u32) -> U256 {
    debug_assert!(shift < N_UDWORD_BITS);
    let mut low = u[0] >> shift;
    let mut high = u[1] >> shift;
    if shift > 0 {
        let sh = N_UDWORD_BITS - shift;
        low |= u[1] << sh;
        high |= u[2] << sh;
    }

    U256::from_u128(low, high)
}

// returns (lo, hi)
#[inline]
const fn split_u128_to_u128(a: u128) -> (u128, u128) {
    (a & 0xFFFFFFFFFFFFFFFF, a >> (N_UDWORD_BITS / 2))
}

// returns (lo, hi)
#[inline]
const fn fullmul_u128(a: u128, b: u128) -> (u128, u128) {
    let (a0, a1) = split_u128_to_u128(a);
    let (b0, b1) = split_u128_to_u128(b);

    let mut t = a0 * b0;
    let mut k: u128;
    let w3: u128;
    (w3, k) = split_u128_to_u128(t);

    t = a1 * b0 + k;
    let (w1, w2) = split_u128_to_u128(t);
    t = a0 * b1 + w1;
    k = t >> 64;

    let w_hi = a1 * b1 + w2 + k;
    let w_lo = (t << 64) + w3;

    (w_lo, w_hi)
}

#[inline]
fn fullmul_u256_u128(a: &U256, b: u128) -> [u128; 3] {
    let mut acc = [0_u128; 3];
    let mut lo: u128;
    let mut carry: u128;
    let c: bool;
    if b != 0 {
        (lo, carry) = fullmul_u128(a.low(), b);
        acc[0] = lo;
        acc[1] = carry;
        (lo, carry) = fullmul_u128(a.high(), b);
        (acc[1], c) = acc[1].overflowing_add(lo);
        acc[2] = carry + c as u128;
    }

    acc
}

#[inline]
const fn add_carry(a: u128, b: u128, c: bool) -> (u128, bool) {
    let (res1, overflow1) = b.overflowing_add(c as u128);
    let (res2, overflow2) = u128::overflowing_add(a, res1);

    (res2, overflow1 || overflow2)
}

#[inline]
const fn sub_carry(a: u128, b: u128, c: bool) -> (u128, bool) {
    let (res1, overflow1) = b.overflowing_add(c as u128);
    let (res2, overflow2) = u128::overflowing_sub(a, res1);

    (res2, overflow1 || overflow2)
}

#[inline]
fn knuth_div_mod(u: &U256, v: &U256) -> (U256, U256) {
    // D1.
    // Make sure 128th bit in v's highest word is set.
    // If we shift both u and v, it won't affect the quotient
    // and the remainder will only need to be shifted back.
    let shift = v.high().leading_zeros();
    debug_assert!(shift < N_UDWORD_BITS);
    let v = *v << shift;
    debug_assert!(v.high() >> (N_UDWORD_BITS - 1) == 1);
    // u will store the remainder (shifted)
    let mut u = full_shl(u, shift);

    // quotient
    let v_n_1 = v.high();
    let v_n_2 = v.low();

    // D2. D7. - unrolled loop j == 0, n == 2, m == 0 (only one possible iteration)
    let mut r_hat: u128 = 0;
    let u_jn = u[2];

    // D3.
    // q_hat is our guess for the j-th quotient digit
    // q_hat = min(b - 1, (u_{j+n} * b + u_{j+n-1}) / v_{n-1})
    // b = 1 << WORD_BITS
    // Theorem B: q_hat >= q_j >= q_hat - 2
    let mut q_hat = if u_jn < v_n_1 {
        //let (mut q_hat, mut r_hat) = _div_mod_u128(u_jn, u[j + n - 1], v_n_1);
        let mut q_hat = udiv256_by_128_to_128(u_jn, u[1], v_n_1, &mut r_hat);
        let mut overflow: bool;
        // this loop takes at most 2 iterations
        loop {
            let another_iteration = {
                // check if q_hat * v_{n-2} > b * r_hat + u_{j+n-2}
                let (lo, hi) = fullmul_u128(q_hat, v_n_2);
                hi > r_hat || (hi == r_hat && lo > u[0])
            };
            if !another_iteration {
                break;
            }
            q_hat -= 1;
            (r_hat, overflow) = r_hat.overflowing_add(v_n_1);
            // if r_hat overflowed, we're done
            if overflow {
                break;
            }
        }
        q_hat
    } else {
        // here q_hat >= q_j >= q_hat - 1
        u128::MAX
    };

    // ex. 20:
    // since q_hat * v_{n-2} <= b * r_hat + u_{j+n-2},
    // either q_hat == q_j, or q_hat == q_j + 1

    // D4.
    // let's assume optimistically q_hat == q_j
    // subtract (q_hat * v) from u[j..]
    let q_hat_v = fullmul_u256_u128(&v, q_hat);
    // u[j..] -= q_hat_v;
    let mut c = false;
    (u[0], c) = sub_carry(u[0], q_hat_v[0], c);
    (u[1], c) = sub_carry(u[1], q_hat_v[1], c);
    (u[2], c) = sub_carry(u[2], q_hat_v[2], c);

    // D6.
    // actually, q_hat == q_j + 1 and u[j..] has overflowed
    // highly unlikely ~ (1 / 2^127)
    if c {
        q_hat -= 1;
        // add v to u[j..]
        c = false;
        (u[0], c) = add_carry(u[0], v.low(), c);
        (u[1], c) = add_carry(u[1], v.high(), c);
        u[2] = u[2].wrapping_add(c as u128);
    }

    // D5.
    // let mut q = U256::ZERO;
    // *q.low_mut() = q_hat;
    let q = U256::from_u128(q_hat, 0);

    // D8.
    let remainder = full_shr(&u, shift);

    (q, remainder)
}

impl From<u128> for U256 {
    #[inline(always)]
    fn from(val: u128) -> U256 {
        U256 { high: 0, low: val }
    }
}

impl From<u64> for U256 {
    #[inline(always)]
    fn from(val: u64) -> Self {
        U256 {
            high: 0,
            low: val as u128,
        }
    }
}

impl PartialEq<u128> for U256 {
    #[inline(always)]
    fn eq(&self, other: &u128) -> bool {
        self.eq(&U256::from(*other))
    }
}

impl PartialOrd<u128> for U256 {
    #[inline(always)]
    fn partial_cmp(&self, other: &u128) -> Option<Ordering> {
        self.partial_cmp(&U256::from(*other))
    }
}

impl Add for U256 {
    type Output = U256;

    #[inline(always)]
    fn add(self, other: Self) -> U256 {
        let (res, overflow) = self.overflowing_add(other);
        assert!(!overflow, "U256 add overflow");
        res
    }
}

impl Add<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn add(self, other: u128) -> U256 {
        self.add(U256::from(other))
    }
}

impl Sub<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn sub(self, other: u128) -> U256 {
        let (low, borrow) = self.low().overflowing_sub(other);
        let (high, overflow) = self.high().overflowing_sub(borrow as u128);
        assert!(!overflow, "U256 sub overflows");
        U256 { high, low }
    }
}

impl Sub<U256> for u128 {
    type Output = u128;

    #[inline]
    fn sub(self, other: U256) -> u128 {
        assert_eq!(other.high(), 0, "U256 sub overflows");

        let (result, overflow) = self.overflowing_sub(other.low());
        assert!(!overflow, "U256 sub overflows");
        result
    }
}

impl Mul for U256 {
    type Output = U256;

    #[inline(always)]
    fn mul(self, other: Self) -> U256 {
        let (res, overflow) = self.overflowing_mul(other);
        assert!(!overflow, "U256 mul overflows");
        res
    }
}

impl Mul<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn mul(self, other: u128) -> U256 {
        self.mul(U256::from(other))
    }
}

impl Div for U256 {
    type Output = U256;

    #[inline(always)]
    fn div(self, other: U256) -> U256 {
        self.div_rem(other).0
    }
}

impl Div<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn div(self, other: u128) -> U256 {
        self.div(U256::from(other))
    }
}

impl Rem for U256 {
    type Output = U256;

    #[inline(always)]
    fn rem(self, other: U256) -> U256 {
        self.div_rem(other).1
    }
}

impl Rem<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn rem(self, other: u128) -> U256 {
        self.div_rem(other).1
    }
}

impl Rem<U256> for u128 {
    type Output = U256;

    #[inline(always)]
    fn rem(self, other: U256) -> U256 {
        U256::from(self).rem(other)
    }
}

impl Shl<u32> for U256 {
    type Output = U256;

    fn shl(self, rhs: u32) -> U256 {
        debug_assert!(rhs < 256, "shl intrinsic called with overflowing shift");

        let (hi, lo) = if rhs == 0 {
            return self;
        } else if rhs < 128 {
            ((self.high() << rhs) | (self.low() >> (128 - rhs)), self.low() << rhs)
        } else {
            (self.low() << (rhs & 0x7f), 0)
        };

        U256::from_u128(lo, hi)
    }
}

impl Shr<u32> for U256 {
    type Output = U256;

    fn shr(self, rhs: u32) -> U256 {
        debug_assert!(rhs < 256, "shr intrinsic called with overflowing shift");

        let (hi, lo) = if rhs == 0 {
            return self;
        } else if rhs < 128 {
            (self.high() >> rhs, self.low() >> rhs | (self.high() << (128 - rhs)))
        } else {
            (0, self.high() >> (rhs & 0x7f))
        };

        U256::from_u128(lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_powers_10() {
        let mut val = U256::from(1u64);
        let ten = U256::from(10u64);
        for i in 0..77 {
            if i != 0 {
                val = val.wrapping_mul(ten);
            };
            println!("U256::from_u128({}, {}),", val.low(), val.high(),);
        }
    }

    #[test]
    fn generate_roundings() {
        let mut val = U256::from(5u64);
        let ten = U256::from(10u64);
        for i in 0..77 {
            let v = if i == 0 {
                U256::ZERO
            } else if i == 1 {
                val
            } else {
                val = val.wrapping_mul(ten);
                val
            };
            println!("U256::from_u128({}, {}),", v.low(), v.high(),);
        }
    }

    #[test]
    fn test_powers_ten() {
        let mut prev = POWERS_10[0];
        assert_eq!(prev, 1u128);
        for &val in POWERS_10.iter().skip(1) {
            assert!(val > prev);
            assert_eq!(val.cmp(&prev), Ordering::Greater);
            let div = val / prev;
            assert_eq!(div, 10u128);
            prev = val;
        }
    }

    #[test]
    fn test_count_digits() {
        fn assert(val: U256, count_digits: u32) {
            assert_eq!(val.count_digits(), count_digits);
        }

        assert(U256::from(0u128), 1);
        assert(U256::from(1u128), 1);
        assert(U256::from(10u128), 2);
        assert(U256::from(11u128), 2);
        assert(U256::from(99u128), 2);
        assert(U256::from(100u128), 3);
        assert(U256::from(1_0000_0001u128), 9);
        assert(U256::from(1_0000_0000_0001u128), 13);
        assert(U256::from(1_0000_0000_0000_0000u128), 17);
        assert(U256::from(1_0000_0000_0000_0000_0000_0000_0000u128), 29);
        assert(U256::from(10_0000_0000_0000_0000_0000_0000_0000_0000_0000u128), 38);
        assert(U256::from(100_0000_0000_0000_0000_0000_0000_0000_0000_0000u128), 39);
        assert(
            U256::mul128(100_0000_0000_0000_0000_0000_0000, 1_0000_0000_0000_0000_0000),
            47,
        );
    }

    #[test]
    fn test_add() {
        assert_eq!(U256::from(u128::MAX) + 1, U256::from_u128(0, 1));
        assert_eq!(
            U256::from(u128::MAX) + U256::from(u128::MAX),
            U256::from_u128(u128::MAX - 1, 1)
        );
        assert_eq!(U256::from(1u128) + U256::from(1u128), U256::from(2u128));
        assert_eq!(U256::from_u128(1, 1) + U256::from_u128(1, 1), U256::from_u128(2, 2));
        assert_eq!(
            U256::from_u128(1, 1) + U256::from_u128(u128::MAX, 1),
            U256::from_u128(0, 3)
        );

        // overflow
        assert!(U256::from_u128(0, u128::MAX).overflowing_add(U256::from_u128(0, 1)).1);
        assert!(
            U256::from_u128(1, u128::MAX)
                .overflowing_add(U256::from_u128(u128::MAX, 0))
                .1
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(U256::from_u128(0, 1) - 1, U256::from(u128::MAX));
        assert_eq!(U256::from_u128(0, 1) - u128::MAX, U256::from(1u128));
        assert_eq!(U256::from_u128(1, 1) - 1, U256::from_u128(0, 1));
        assert_eq!(2 - U256::from(1u128), 1);

        // overflow
        assert!(U256::from_u128(1, 1).overflowing_sub(U256::from_u128(1, 2)).1);
        assert!(U256::from_u128(1, 1).overflowing_sub(U256::from_u128(2, 1)).1);
    }

    #[test]
    fn test_mul() {
        assert_eq!(U256::from_u128(0, 1) * 2, U256::from_u128(0, 2));
        assert_eq!(U256::from_u128(1, 1) * 2, U256::from_u128(2, 2));
        assert_eq!(U256::from_u128(u128::MAX, 1) * 2, U256::from_u128(u128::MAX - 1, 3));
        assert_eq!(U256::mul128(u128::MAX, u128::MAX), U256::from_u128(1, u128::MAX - 1));
        assert_eq!(
            U256::mul128(u64::MAX as u128, u64::MAX as u128 + 2),
            U256::from_u128(u128::MAX, 0)
        );

        // overflow
        assert!(U256::from_u128(0, 1).overflowing_mul(U256::from_u128(0, 1)).1);
        assert!(U256::from_u128(2, 1).overflowing_mul(U256::from(u128::MAX)).1);
    }

    #[test]
    fn test_div_mod() {
        assert_eq!(U256::from_u128(3, 0) / U256::from_u128(2, 0), U256::from(1u128));
        assert_eq!(U256::from_u128(3, 0) % U256::from_u128(2, 0), U256::from(1u128));
        assert_eq!(U256::from_u128(0, 3) / U256::from_u128(0, 2), U256::from(1u128));
        assert_eq!(U256::from_u128(0, 3) % U256::from_u128(0, 2), U256::from_u128(0, 1));
        assert_eq!(
            U256::from_u128(0, 3) / U256::from_u128(2, 0),
            U256::from_u128(1 << 127, 1)
        );
        assert_eq!(
            U256::from_u128(0, 3) / U256::from_u128(4, 0),
            U256::from_u128(3 << 126, 0)
        );
        assert_eq!(U256::from_u128(0, 1) / U256::from_u128(0, 2), U256::from(0u128));
        assert_eq!(U256::from_u128(0, 1) % U256::from_u128(0, 2), U256::from_u128(0, 1));
        assert_eq!(
            U256::from_u128(
                736215297081859961199755827885906233,
                214064674252647095149109719693322140207
            ) / U256::from(320000000000000000000000000000000000000u128),
            U256::from(227632606340157585901208756549081254077u128)
        );
    }
}

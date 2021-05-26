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
use std::mem::MaybeUninit;
use std::ops::{Add, Div, Mul, Rem, Sub};

pub const POWERS_10: [U256; (MAX_PRECISION * 2 + 1) as usize] = [
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
    U256::from_u128(
        62657657327783134786937801511322779648,
        293873587705571876992,
    ),
    U256::from_u128(
        286294206356892884406003407681459585024,
        2938735877055718769921,
    ),
    U256::from_u128(
        140683128201421136353037217360450158592,
        29387358770557187699218,
    ),
    U256::from_u128(
        45701814330457509676873743877428740096,
        293873587705571876992184,
    ),
    U256::from_u128(
        116735776383636633305362831342519189504,
        2938735877055718769921841,
    ),
    U256::from_u128(
        146510663073550942663504491129887260672,
        29387358770557187699218413,
    ),
    U256::from_u128(
        103977163051755572781546481571799760896,
        293873587705571876992184134,
    ),
    U256::from_u128(
        18924529754740337425340993422692974592,
        2938735877055718769921841343,
    ),
    U256::from_u128(
        189245297547403374253409934226929745920,
        29387358770557187699218413430,
    ),
    U256::from_u128(
        191041140869341425217226305110456401920,
        293873587705571876992184134305,
    ),
    U256::from_u128(
        208999574088721934855390013945722961920,
        2938735877055718769921841343055,
    ),
    U256::from_u128(
        48301539361588567773652494866620350464,
        29387358770557187699218413430556,
    ),
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

pub const ROUNDINGS: [U256; (MAX_PRECISION * 2 + 1) as usize] = [
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
    U256::from_u128(
        207302303018952234817371654534627065856,
        14693679385278593849,
    ),
    U256::from_u128(
        31328828663891567393468900755661389824,
        146936793852785938496,
    ),
    U256::from_u128(
        313288286638915673934689007556613898240,
        1469367938527859384960,
    ),
    U256::from_u128(
        70341564100710568176518608680225079296,
        14693679385278593849609,
    ),
    U256::from_u128(
        22850907165228754838436871938714370048,
        146936793852785938496092,
    ),
    U256::from_u128(
        228509071652287548384368719387143700480,
        1469367938527859384960920,
    ),
    U256::from_u128(
        243396514997244703063439549280827736064,
        14693679385278593849609206,
    ),
    U256::from_u128(
        51988581525877786390773240785899880448,
        146936793852785938496092067,
    ),
    U256::from_u128(
        179603448337839400444357800427230593024,
        1469367938527859384960920671,
    ),
    U256::from_u128(
        94622648773701687126704967113464872960,
        14693679385278593849609206715,
    ),
    U256::from_u128(
        265661753895139944340300456271112306688,
        146936793852785938496092067152,
    ),
    U256::from_u128(
        274640970504830199159382310688745586688,
        1469367938527859384960920671527,
    ),
    U256::from_u128(
        24150769680794283886826247433310175232,
        14693679385278593849609206715278,
    ),
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct U256(ethnum::U256);

impl U256 {
    pub const ZERO: U256 = U256::from_u128(0, 0);
    pub const ONE: U256 = U256::from_u128(1, 0);

    #[inline(always)]
    pub const fn from_u128(low: u128, high: u128) -> U256 {
        U256(ethnum::U256::from_words(high, low))
    }

    #[inline(always)]
    pub fn low(&self) -> u128 {
        *self.0.low()
    }

    #[inline(always)]
    pub fn high(&self) -> u128 {
        *self.0.high()
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
    pub fn overflowing_add<T: Into<U256>>(&self, other: T) -> (U256, bool) {
        let (add, overflow) = self.0.overflowing_add(other.into().0);
        (U256(add), overflow)
    }

    #[inline(always)]
    pub fn overflowing_sub<T: Into<U256>>(&self, other: T) -> (U256, bool) {
        let (add, overflow) = self.0.overflowing_sub(other.into().0);
        (U256(add), overflow)
    }

    #[inline(always)]
    pub fn checked_mul(&self, other: U256) -> Option<U256> {
        self.0.checked_mul(other.0).map(U256)
    }

    #[inline(always)]
    pub fn wrapping_mul(&self, other: U256) -> U256 {
        U256(self.0.wrapping_mul(other.0))
    }

    #[inline]
    pub fn div_rem<T: Into<U256>>(&self, other: T) -> (U256, U256) {
        let other = other.into();
        let mut result = MaybeUninit::uninit();
        let mut remain = MaybeUninit::uninit();
        ethnum::intrinsics::udivmod4(&mut result, &self.0, &other.0, Some(&mut remain));
        unsafe { (U256(result.assume_init()), U256(remain.assume_init())) }
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
        self.0.partial_cmp(&other).unwrap()
    }

    #[inline(always)]
    pub fn add128(left: u128, right: u128) -> U256 {
        U256::from(left) + U256::from(right)
    }

    #[inline(always)]
    pub fn mul128(left: u128, right: u128) -> U256 {
        U256(ethnum::intrinsics::mulddi3(&left, &right))
    }
}

impl From<ethnum::U256> for U256 {
    #[inline(always)]
    fn from(val: ethnum::U256) -> Self {
        U256(val)
    }
}

impl From<u128> for U256 {
    #[inline(always)]
    fn from(val: u128) -> U256 {
        U256(ethnum::U256::from(val))
    }
}

impl From<u64> for U256 {
    #[inline(always)]
    fn from(val: u64) -> Self {
        U256(ethnum::U256::from(val))
    }
}

impl PartialEq<u128> for U256 {
    #[inline(always)]
    fn eq(&self, other: &u128) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u128> for U256 {
    #[inline(always)]
    fn partial_cmp(&self, other: &u128) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl Add for U256 {
    type Output = U256;

    #[inline(always)]
    fn add(self, other: Self) -> U256 {
        U256(self.0.add(other.0))
    }
}

impl Add<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn add(self, other: u128) -> U256 {
        U256(self.0.add(other))
    }
}

impl Sub<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn sub(self, other: u128) -> U256 {
        U256(self.0.sub(other))
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
        U256(self.0.mul(other.0))
    }
}

impl Mul<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn mul(self, other: u128) -> U256 {
        U256(self.0.mul(other))
    }
}

impl Div for U256 {
    type Output = U256;

    #[inline(always)]
    fn div(self, other: U256) -> U256 {
        U256(self.0.div(other.0))
    }
}

impl Div<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn div(self, other: u128) -> U256 {
        U256(self.0.div(other))
    }
}

impl Rem<u128> for U256 {
    type Output = U256;

    #[inline(always)]
    fn rem(self, other: u128) -> U256 {
        U256(self.0.rem(other))
    }
}

impl Rem<U256> for u128 {
    type Output = U256;

    #[inline(always)]
    fn rem(self, other: U256) -> U256 {
        U256(ethnum::U256::from(self).rem(other.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn generate_powers_10() {
        let mut val = U256::from(1u64);
        let ten = U256::from(10u64);
        for i in 0..77 {
            let v = if i == 0 {
                val
            } else {
                val = val.wrapping_mul(ten);
                val
            };
            println!("U256::from_u128({}, {}),", v.low(), v.high(),);
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
        assert(
            U256::from(10_0000_0000_0000_0000_0000_0000_0000_0000_0000u128),
            38,
        );
        assert(
            U256::from(100_0000_0000_0000_0000_0000_0000_0000_0000_0000u128),
            39,
        );
        assert(
            U256(
                ethnum::U256::from_str("10000000000000000000000000000000000000000000000").unwrap(),
            ),
            47,
        );
    }
}

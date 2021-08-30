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

//! High precision decimal with maximum precision of 38.
//!
//! ## Optional features
//!
//! ### `serde`
//!
//! When this optional dependency is enabled, `Decimal` implements the `serde::Serialize` and
//! `serde::Deserialize` traits.
//!
//! ## Usage
//!
//! To build a decimal, use [`Decimal`]:
//!
//! ```
//! use decimal_rs::Decimal;
//!
//! let n1: Decimal = "123".parse().unwrap();
//! let n2: Decimal = "456".parse().unwrap();
//! let result = n1 + n2;
//! assert_eq!(result.to_string(), "579");
//! ```
//!
//! To build a decimal from Rust primitive types:
//!
//! ```
//! use decimal_rs::Decimal;
//!
//! let n1 = Decimal::from(123_i32);
//! let n2 = Decimal::from(456_i32);
//! let result = n1 + n2;
//! assert_eq!(result, Decimal::from(579_i32));
//! ```
//!
//! Decimal supports high precision arithmetic operations.
//!
//! ```
//! use decimal_rs::Decimal;
//!
//! let n1: Decimal = "123456789.987654321".parse().unwrap();
//! let n2: Decimal = "987654321.123456789".parse().unwrap();
//! let result = n1 * n2;
//! assert_eq!(result.to_string(), "121932632103337905.662094193112635269");
//! ```
//!
//! Decimal can be encoded to bytes and decoded from bytes.
//!
//! ```
//! use decimal_rs::Decimal;
//!
//! let n1 = "123456789.987654321".parse::<Decimal>().unwrap();
//! let mut  bytes = Vec::new();
//! n1.encode(&mut bytes).unwrap();
//! let n2 = Decimal::decode(&bytes);
//! assert_eq!(n1, n2);
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

mod convert;
mod decimal;
mod error;
mod ops;
mod parse;
mod u256;

#[cfg(feature = "serde")]
mod serde;

pub use crate::decimal::{Decimal, MAX_BINARY_SIZE, MAX_PRECISION};
pub use crate::error::{DecimalConvertError, DecimalFormatError, DecimalParseError};

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

//! Decimal error definitions.

use std::num::ParseFloatError;
use thiserror::Error;

/// An error which can be returned when parsing a decimal.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DecimalParseError {
    /// Empty string.
    #[error("cannot parse number from empty string")]
    Empty,
    /// Invalid decimal.
    #[error("invalid number")]
    Invalid,
    /// Decimal is overflowed.
    #[error("value overflows number format")]
    Overflow,
}

/// An error which can be returned when a conversion between other type and decimal fails.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum DecimalConvertError {
    /// Invalid decimal.
    #[error("invalid number")]
    Invalid,
    /// Decimal is overflowed.
    #[error("numeric overflow")]
    Overflow,
}

impl From<DecimalParseError> for DecimalConvertError {
    #[inline]
    fn from(e: DecimalParseError) -> Self {
        match e {
            DecimalParseError::Empty | DecimalParseError::Invalid => DecimalConvertError::Invalid,
            DecimalParseError::Overflow => DecimalConvertError::Overflow,
        }
    }
}

impl From<ParseFloatError> for DecimalConvertError {
    #[inline]
    fn from(_: ParseFloatError) -> Self {
        DecimalConvertError::Invalid
    }
}

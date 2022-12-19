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

use std::fmt;
use std::num::ParseFloatError;

/// An error which can be returned when parsing a decimal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecimalParseError {
    /// Empty string.
    Empty,
    /// Invalid decimal.
    Invalid,
    /// Decimal is overflowed.
    Overflow,
    /// Decimal is underflow.
    Underflow,
}

impl fmt::Display for DecimalParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            DecimalParseError::Empty => write!(f, "cannot parse number from empty string"),
            DecimalParseError::Invalid => write!(f, "invalid number"),
            DecimalParseError::Overflow => write!(f, "numeric overflow"),
            DecimalParseError::Underflow => write!(f, "numeric underflow"),
        }
    }
}

/// An error which can be returned when a conversion between other type and decimal fails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecimalConvertError {
    /// Invalid decimal.
    Invalid,
    /// Decimal is overflowed.
    Overflow,
}

impl fmt::Display for DecimalConvertError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            DecimalConvertError::Invalid => write!(f, "invalid number"),
            DecimalConvertError::Overflow => write!(f, "numeric overflow"),
        }
    }
}

/// An error which can be returned when format decimal to string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecimalFormatError {
    /// std::fmt::Error
    Format(fmt::Error),
    /// Decimal is out of range.
    OutOfRange,
}

impl std::error::Error for DecimalFormatError {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            DecimalFormatError::Format(e) => Some(e),
            DecimalFormatError::OutOfRange => None,
        }
    }
}

impl fmt::Display for DecimalFormatError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            DecimalFormatError::Format(e) => write!(f, "{}", e),
            DecimalFormatError::OutOfRange => write!(f, "Data value out of range"),
        }
    }
}

impl From<DecimalParseError> for DecimalConvertError {
    #[inline]
    fn from(e: DecimalParseError) -> Self {
        match e {
            DecimalParseError::Empty | DecimalParseError::Invalid => DecimalConvertError::Invalid,
            DecimalParseError::Overflow | DecimalParseError::Underflow => DecimalConvertError::Overflow,
        }
    }
}

impl From<ParseFloatError> for DecimalConvertError {
    #[inline]
    fn from(_: ParseFloatError) -> Self {
        DecimalConvertError::Invalid
    }
}

impl From<fmt::Error> for DecimalFormatError {
    #[inline]
    fn from(e: fmt::Error) -> Self {
        DecimalFormatError::Format(e)
    }
}

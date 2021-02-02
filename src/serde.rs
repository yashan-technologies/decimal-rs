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

//! serde implementation.

use crate::buf::Buf;
use crate::Decimal;

impl serde::Serialize for Decimal {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use std::io::Write;

        let mut buf = Buf::new();
        if serializer.is_human_readable() {
            write!(&mut buf, "{}", self).map_err(serde::ser::Error::custom)?;
            let str = unsafe { std::str::from_utf8_unchecked(buf.as_slice()) };
            str.serialize(serializer)
        } else {
            self.encode(&mut buf).map_err(serde::ser::Error::custom)?;
            buf.as_slice().serialize(serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for Decimal {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct DecimalVisitor;

        impl<'de> serde::de::Visitor<'de> for DecimalVisitor {
            type Value = Decimal;

            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a decimal")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Decimal, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(serde::de::Error::custom)
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Decimal, E>
            where
                E: serde::de::Error,
            {
                let n = Decimal::decode(v);
                Ok(n)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(DecimalVisitor)
        } else {
            deserializer.deserialize_bytes(DecimalVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let dec = "123.456".parse::<Decimal>().unwrap();

        let json = serde_json::to_string(&dec).unwrap();
        assert_eq!(json, r#""123.456""#);
        let json_dec: Decimal = serde_json::from_str(&json).unwrap();
        assert_eq!(json_dec, dec);

        let bin = bincode::serialize(&dec).unwrap();
        let bin_dec: Decimal = bincode::deserialize(&bin).unwrap();
        assert_eq!(bin_dec, dec);
    }
}

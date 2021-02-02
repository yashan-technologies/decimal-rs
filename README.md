# decimal-rs

[![Apache-2.0 licensed](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Crate](https://img.shields.io/crates/v/decimal-rs.svg)](https://crates.io/crates/decimal-rs)
[![API](https://docs.rs/decimal-rs/badge.svg)](https://docs.rs/decimal-rs)

High precision decimal with maximum precision of 38.

## Optional features

### `serde`

When this optional dependency is enabled, `Decimal` implements the `serde::Serialize` and `serde::Deserialize` traits.

## Usage

To build a decimal, use `Decimal`:

```Rust
use decimal_rs::Decimal;

let n1: Decimal = "123".parse().unwrap();
let n2: Decimal = "456".parse().unwrap();
let result = n1 + n2;
assert_eq!(result.to_string(), "579");
```

To build a decimal from Rust primitive types:

```Rust
use decimal_rs::Decimal;

let n1 = Decimal::from(123_i32);
let n2 = Decimal::from(456_i32);
let result = n1 + n2;
assert_eq!(result, Decimal::from(579_i32));
```

Decimal supports high precision arithmetic operations.

```Rust
use decimal_rs::Decimal;

let n1: Decimal = "123456789.987654321".parse().unwrap();
let n2: Decimal = "987654321.123456789".parse().unwrap();
let result = n1 * n2;
assert_eq!(result.to_string(), "121932632103337905.662094193112635269");
```

Decimal can be encoded to bytes and decoded from bytes.

```Rust
use decimal_rs::Decimal;

let n1 = "123456789.987654321".parse::<Decimal>().unwrap();
let mut  bytes = Vec::new();
n1.encode(&mut bytes).unwrap();
let n2 = Decimal::decode(&bytes);
assert_eq!(n1, n2);
```

## License

This project is licensed under the Apache-2.0 license ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `decimal-rs` by you, shall be licensed as Apache-2.0, without any additional
terms or conditions.

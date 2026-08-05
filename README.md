[![](https://img.shields.io/badge/made%20by-Cryptid%20Technologies-gold.svg?style=flat-square)][CRYPTID]
[![](https://img.shields.io/badge/project-provenance-purple.svg?style=flat-square)][PROVENANCE]
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)][MULTIFORMATS]
![](https://github.com/cryptidtech/multiutil/actions/workflows/rust.yml/badge.svg)

# multi-util

Traits, types, and functions to construct multiformat types.

## Features

- `no_std` support. The crate works in `no_std` environments with `alloc`.
- Zero unsafe code. `#![deny(unsafe_code)]` is set at the crate root.
- DoS protection. The `Varbytes` decode path enforces a decoded-size cap and a buffer-length check.
- Configurable decoded-size cap. Use `VarbytesMax<N>` to set a custom cap at the type level. The default `Varbytes` alias uses 16 MiB.

## Install

Add this to your `Cargo.toml`:

```toml
[dependencies]
multi-util = "1.1"
```

For `no_std` environments, disable `std` and `serde`:

```toml
[dependencies]
multi-util = { version = "1.1", default-features = false }
```

To use serde under `no_std`, enable only the `serde` feature:

```toml
[dependencies]
multi-util = { version = "1.1", default-features = false, features = ["serde"] }
```

MSRV: Rust 1.85 (Edition 2024).

## Feature Flags

- `std` (default). Enables `std` support. It pulls in the `std`-gated features of `multi-base`, `multi-codec`, `multi-trait`, `thiserror`, and `serde`.
- `serde` (default). Enables serde serialization and deserialization for `BaseEncoded`, `Varuint`, and `Varbytes`.

## BaseEncoded

The `BaseEncoded` smart pointer wraps any multiformat type that implements the `EncodingInfo` trait. `BaseEncoded` handles base encoding of the inner value. It uses the [Multibase][MULTIBASE] text encoding systems.

## CodecInfo

The `CodecInfo` trait lets a multiformat type expose its [Multicodec][MULTICODEC] value to code that uses this trait.

## Varuint

`Varuint` is an implementation of a [variable length, unsigned integer][VARUINT]. It is common to all multiformat protocols and types.

## Varbytes

`Varbytes` is a `Varuint` followed by a binary octet array of equal length. It is a common way to encode arbitrary binary data. Any code can skip over the data if it does not know how, or does not want, to process it.

```
<varbytes> ::= <varuint> N(OCTET)
                    ^        ^
                   /          \
           count of            variable number
             octets            of octets
```

### Decoded-Size Caps

`Varbytes::try_decode_from` and the serde `Varbytes` path enforce two caps on untrusted wire data:

- `DEFAULT_MAX = 16 MiB`. This is the most bytes a single decoded `Varbytes` value can allocate. If the length prefix claims more, the decode returns `Error::InputTooLarge`.
- Buffer-length check. If the length prefix claims more bytes than remain in the buffer, the decode returns `Error::InsufficientData`. This prevents an out-of-bounds read.

The serde path routes all visitor impls through a shared `decode_varbytes` helper. The helper runs both checks before it slices the buffer.

### Configurable Cap (A4)

The decoded-size cap is configurable at the type level. `VarbytesMax<const MAX: usize>` is the generic struct. `Varbytes` is a type alias for `VarbytesMax<DEFAULT_MAX>` (16 MiB). To set a custom cap, instantiate `VarbytesMax<N>` directly:

```rust
use multi_util::varbytes::VarbytesMax;

// A Varbytes that rejects payloads over 1024 bytes.
type SmallVarbytes = VarbytesMax<1024>;
```

The `MAX` const generic sets the maximum decoded size. `try_decode_from` and the serde `Deserialize` impl both enforce it. The crate exports `deserialize_varbytes_with_max(deserializer, max)` for a per-field override without a distinct type.

## Security

See [SECURITY.md](SECURITY.md) for the full security policy.

- `#![deny(unsafe_code)]` is set at the crate root.
- All errors return `Result`. No path panics on invalid input.
- The `Varbytes` decode path enforces `DEFAULT_MAX` (16 MiB) and a buffer-length check. This mitigates CWE-400 and CWE-125.

[CRYPTID]: https://cryptid.tech/
[PROVENANCE]: https://github.com/cryptidtech/provenance-specifications/
[MULTIFORMATS]: https://github.com/multiformats/multiformats/
[MULTIBASE]: https://github.com/multiformats/multibase
[MULTICODEC]: https://github.com/multiformats/multicodec
[VARUINT]: https://github.com/multiformats/unsigned-varint
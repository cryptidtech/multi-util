# Security Policy

## Overview

The `multi-util` crate gives utility traits and types to the multiformats stack. These types are `BaseEncoded`, `Varuint`, `Varbytes`, `CodecInfo`, and `EncodingInfo`. This document describes the security properties of the crate.

## `no_std` Support

The crate works in `no_std` environments with `alloc`. Disable the default features to remove the `std` dependency:

```toml
[dependencies]
multi-util = { version = "1.1", default-features = false }
```

To use serde under `no_std`, enable only the `serde` feature:

```toml
[dependencies]
multi-util = { version = "1.1", default-features = false, features = ["serde"] }
```

The `std` feature is on by default. It enables `multi-base/std`, `multi-codec/std`, `multi-trait/std`, `thiserror/std`, and `serde?/std`. A CI `ensure_no_std` job builds the `no_std` target on each push and pull request.

## Memory Safety

- No unsafe code. `#![deny(unsafe_code)]` is set at the crate root. `[lints.rust] unsafe_code = "deny"` in `Cargo.toml` enforces it too.
- All errors return `Result`. No path panics on invalid input.

## Decoded-Size Caps

`Varbytes::try_decode_from` and the serde `Varbytes` path enforce two caps on untrusted wire data. These caps mitigate CWE-400 and CWE-125.

- `DEFAULT_MAX = 16 MiB`. This is the most bytes a single decoded `Varbytes` value can allocate. A length prefix that claims more returns `Error::InputTooLarge { claimed, max }`.
- Buffer-length check. A length prefix that claims more bytes than remain returns `Error::InsufficientData { expected, actual }`. This prevents an out-of-bounds read and a panic.

The serde path routes all four visitor impls through a shared `decode_varbytes(input, max)` helper. The helper runs both checks before it slices the buffer.

### Configurable Cap (A4)

The decoded-size cap is configurable at the type level. `VarbytesMax<const MAX: usize>` is the generic struct. `Varbytes` is a type alias for `VarbytesMax<DEFAULT_MAX>` (16 MiB). A caller that needs a different bound has two options:

1. Use a distinct type. Instantiate `VarbytesMax<N>` with the desired cap. The `try_decode_from` and `Deserialize` impls enforce `N`.
2. Use a per-field override. The crate exports `deserialize_varbytes_with_max(deserializer, max)`. Use it with `#[serde(deserialize_with = "...")]` when a field needs a tighter or looser bound than the default.

The crate keeps `MAX_DECODED_SIZE` as a deprecated alias for `DEFAULT_MAX`. New code should use `DEFAULT_MAX` or `VarbytesMax<N>`.

## Display/Hash Allocation

The `Hash` impl hashes the raw encoded bytes directly. It does not go through `Display` or `String`. This removes the `Display` and `String` allocation overhead. The `Display` impl keeps one `Vec<u8>` allocation. It needs `&[u8]` to pass the bytes to the base encoder. A zero-allocation `AsRef<[u8]>` path was not adopted. It would need a breaking bound change on the `T` type parameter of `BaseEncoded`. That change would break `Varuint<T>` because its inner value is not a contiguous byte buffer. It would also need language specialization, which is not stable. The source documents this trade-off in `src/base_encoded.rs`.

## Reporting Vulnerabilities

Report security issues through the GitHub issue tracker. You can also report them privately to the maintainers.
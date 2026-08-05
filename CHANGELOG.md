# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-07-29

### Added

- `no_std` support. The crate now works in `no_std` environments with `alloc`. `src/lib.rs` sets `#![cfg_attr(not(feature = "std"), no_std)]` and `#[cfg(not(feature = "std"))] extern crate alloc;`. The crate root doc gives the `no_std` recipe for both serde-off and serde-on configurations.
- `std` feature gate. `default = ["std", "serde"]`. The `std` feature enables `multi-base/std`, `multi-codec/std`, `multi-trait/std`, `thiserror/std`, and `serde?/std`.
- `no_std` CI job. The `ensure_no_std` job in `.github/workflows/rust.yml` builds for `thumbv6m-none-eabi` with `--no-default-features`.
- Coverage CI job. The `coverage` job uses `cargo-llvm-cov` and uploads the result to Codecov.
- Added `no_std` to the crate keywords.
- Configurable `MAX_DECODED_SIZE` via const-generic `VarbytesMax<const MAX: usize>` with `pub type Varbytes = VarbytesMax<{DEFAULT_MAX}>` for backward compatibility (finding A4). The `DEFAULT_MAX` const (16 MiB) replaces the old hardcoded `MAX_DECODED_SIZE`. The crate keeps `MAX_DECODED_SIZE` as a deprecated alias for `DEFAULT_MAX`. A custom cap is set by instantiating `VarbytesMax<N>` directly. The per-field `deserialize_varbytes_with_max` helper remains for a runtime override without a distinct type.

### Changed

- The `multi-base`, `multi-codec`, and `multi-trait` dependencies now use `default-features = false`. The `std` feature enables their `std` features.
- The `thiserror` dependency now uses `default-features = false`.
- The `serde` dependency changed from `features = ["std"]` to `features = ["alloc", "derive"]`. This lets serde work under `no_std`. The `std` feature gates `serde?/std`.
- Added `#[cfg(not(feature = "std"))] use alloc::...` imports to `src/base_encoded.rs`, `src/base_encoder.rs`, `src/base_util.rs`, `src/error.rs`, `src/varbytes.rs`, `src/varuint.rs`, `src/serde/de.rs`, and `src/serde/ser.rs`. These imports give the crate `Vec`, `String`, `format!`, `vec!`, and `ToString` in `no_std` mode.
- Added a doc comment on the `Error` enum. It was missing before. The absence triggered `missing_docs`.
- Replaced the old `std` Requirement module doc with a `no_std` Support section.
- Rewrote `README.md`, `SECURITY.md`, and `CHANGELOG.md` in ASD-STE100 strict mode. Removed marketing language, passive voice, and long sentences.

### Notes

- P1 (Display/Hash clone). The `Hash` impl for `BaseEncoded` was optimized in 1.0.4. It hashes the raw encoded bytes directly. It does not go through `Display` or `String`. The `Display` impl keeps one `Vec<u8>` clone. A zero-allocation `AsRef<[u8]>` path would need a breaking bound change on the `T` parameter of `BaseEncoded`. That change would break `Varuint<T>` because its inner value is not a contiguous byte buffer. It would also need language specialization, which is not stable. The source documents this trade-off in `src/base_encoded.rs`.

## [1.0.5] - 2026-07-16

### Security

- Removed the unmaintained `serde_cbor` dev-dependency (RUSTSEC-2021-0127). Replaced it with `ciborium` (a maintained CBOR library) in all test code.

### Changed

- Added the `cbor_to_vec` helper function in the serde test module. It wraps `ciborium::into_writer` to replace `serde_cbor::to_vec`.
- Replaced `serde_cbor::from_slice` with `ciborium::from_reader`. The call uses `bytes.as_slice()`, which implements `std::io::Read`.
- Replaced `serde_cbor::to_writer` with `ciborium::into_writer`.
- Replaced `serde_cbor::from_reader` with `ciborium::from_reader`. The API name is the same. The crate is different.
- Changed `test_serde_cbor` from exact-byte comparison to round-trip verification. `ciborium` may encode differently than `serde_cbor`.
- Changed error type annotations from `serde_cbor::Error` to `ciborium::de::Error<std::io::Error>` in bounds-check tests.

### Dependencies

- Removed the `serde_cbor = "0.11"` dev-dependency.
- Added the `ciborium = "0.2"` dev-dependency.
- The dependency count went from 118 to 116 crates.

## [1.0.4] - 2026-07-16

### Security

- Added bounds checks to the serde `Varbytes` deserialization path (`src/serde/de.rs`). All four visitor impls (`visit_borrowed_bytes`, `visit_bytes`, `visit_byte_buf`, `visit_seq`) now route through a shared `decode_varbytes(input, max)` helper. The helper checks `len <= max` and `len <= ptr.len()` before it slices. This prevents panics on crafted input with `len > ptr.len()` and unbounded allocation from large valid `len`. It mirrors the safety checks already in `Varbytes::try_decode_from`.
- Added `pub fn deserialize_varbytes_with_max`. It is a serde `deserialize_with`-compatible function that overrides the default `MAX_DECODED_SIZE` cap for fields that need a tighter or looser bound. Exported via `pub use de::deserialize_varbytes_with_max;`.

### Changed

- `Hash` impl for `BaseEncoded` optimized to hash raw encoded bytes directly. It no longer goes through `Display`/`String` formatting. This removes the `Display` and `String` allocation overhead. It still uses one `Vec<u8>` allocation.
- `Debug` impl for `BaseEncoded` tightened. It removed the unused `Clone + Into<Vec<u8>>` bound.
- Added `impl AsRef<[u8]> for Varbytes` for future zero-alloc paths.
- Reconciled `serde` alloc feature vs std-only crate. Changed the `serde` dependency from `features = ["alloc"]` to `features = ["std"]` to match the crate's std-only reality. Updated the `lib.rs` doc note from `no_std` Note to `std` Requirement.

### Tests

- Added 6 serde bounds-check tests. They are `test_varbytes_serde_len_exceeds_buffer_is_err_not_panic`, `test_varbytes_serde_len_exceeds_buffer_binary`, `test_varbytes_serde_len_exceeds_max_is_err`, `test_varbytes_serde_len_just_under_max_ok`, `test_varbytes_serde_len_just_over_max_is_err`, and `test_varbytes_serde_valid_roundtrip`.

## [1.0.3] - 2026-07-15

### Added

- `#![deny(unsafe_code)]` at the crate root.
- `#[must_use]` on `Error::custom()`.
- MSRV declared. `rust-version = "1.85"` in `Cargo.toml`. CI verifies the MSRV with a dedicated job.
- `cargo audit` job in CI.
- `cargo fmt --check` and `clippy -D warnings` steps in CI.
- Clippy lint configuration. `[lints.clippy]` with `pedantic`, `nursery`, and `cargo` groups set to `warn`. `[lints.rust] unsafe_code = "deny"`.
- `no_std` documentation in the crate root. It notes that the crate is currently std-only. The `serde` alloc feature config does not make the crate `no_std`.
- `Varuint` module documentation. It explains the relationship between `Varuint` and the integer trait impls in `multi-trait`.
- `# Errors` doc section on `BaseEncoder::from_base_encoded`.
- `#[inline]` on `Deref`/`DerefMut` impls. It replaced `#[inline(always)]`.

### Changed

- Edition 2024. Updated from Rust 2021.
- `Varbytes::encode_into`. Replaced `v.append(&mut self.0.clone())` with `v.extend_from_slice(&self.0)`. This avoids cloning the entire payload.
- `Varbytes` serde `Serialize`. Replaced `v.append(&mut self.as_bytes().to_vec())` with `v.extend_from_slice(self.as_bytes())`. This avoids an intermediate allocation.
- `DetectedEncoder::from_base_encoded`. It now bails on the first strict decode success instead of collecting all successful decodings. This avoids O(n) redundant decodes and false positives from overlapping alphabets.
- `DetectedEncoder` doc comment. Fixed typos and formatting.
- Resolved all clippy pedantic, nursery, and cargo warnings across source.

## [1.0.2] - 2026-07-13

### Changed

- Updated `multi-base`, `multi-codec`, and `multi-trait` dependencies from path-based to published crates.io versions.
- Fixed codec name references after the multicodec table sync.

## [1.0.1] - 2026-07-13

### Fixed

- Fixed `use` import formatting in `src/serde/de.rs`, `src/varbytes.rs`, and `src/varuint.rs`. This is cosmetic. The `core::` imports were reformatted.

## [1.0.0] - 2026-07-13

### Changed

- Synced from the bettersign workspace (`bs-multiutil` 0.7.0).
- Renamed the crate from `bs-multiutil` to `multi-util`.
- Initial published release on crates.io as `multi-util`.

[1.1.0]: https://github.com/cryptidtech/multi-util/compare/v1.0.5...v1.1.0
[1.0.5]: https://github.com/cryptidtech/multi-util/compare/v1.0.4...v1.0.5
[1.0.4]: https://github.com/cryptidtech/multi-util/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/cryptidtech/multi-util/compare/v1.0.0...v1.0.3
[1.0.2]: https://github.com/cryptidtech/multi-util/releases/tag/v1.0.2
[1.0.1]: https://github.com/cryptidtech/multi-util/releases/tag/v1.0.1
[1.0.0]: https://github.com/cryptidtech/multi-util/releases/tag/v1.0.0
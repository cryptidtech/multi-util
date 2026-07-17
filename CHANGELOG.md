# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.5] - 2026-07-16

### Security
- Removed unmaintained `serde_cbor` dev-dependency (RUSTSEC-2021-0127). Replaced
  with `ciborium` (a maintained CBOR library) in all test code.

### Changed
- Added `cbor_to_vec` helper function in the serde test module to wrap
  `ciborium::into_writer` (replacing `serde_cbor::to_vec`).
- Replaced `serde_cbor::from_slice` with `ciborium::from_reader` (using
  `bytes.as_slice()` which implements `std::io::Read`).
- Replaced `serde_cbor::to_writer` with `ciborium::into_writer`.
- Replaced `serde_cbor::from_reader` with `ciborium::from_reader` (same API
  name, different crate).
- Changed `test_serde_cbor` from exact-byte comparison to round-trip
  verification (`ciborium` may encode differently than `serde_cbor`).
- Changed error type annotations from `serde_cbor::Error` to
  `ciborium::de::Error<std::io::Error>` in bounds-check tests.

### Dependencies
- Removed `serde_cbor = "0.11"` dev-dependency.
- Added `ciborium = "0.2"` dev-dependency.
- Dependency count reduced from 118 to 116 crates.

## [1.0.4] - 2026-07-16

### Security
- Added bounds checks to serde `Varbytes` deserialization path
  (`src/serde/de.rs`). All four visitor impls (`visit_borrowed_bytes`,
  `visit_bytes`, `visit_byte_buf`, `visit_seq`) now route through a shared
  `decode_varbytes(input, max)` helper that checks `len <= max` and
  `len <= ptr.len()` before slicing — preventing panics on crafted input
  with `len > ptr.len()` and unbounded allocation from large valid `len`.
  Mirrors the safety checks already in `Varbytes::try_decode_from`.
- Added `pub fn deserialize_varbytes_with_max` — a serde
  `deserialize_with`-compatible function that overrides the default
  `MAX_DECODED_SIZE` cap for fields that need a tighter or looser bound.
  Exported via `pub use de::deserialize_varbytes_with_max;`.

### Changed
- `Hash` impl for `BaseEncoded` optimized to hash raw encoded bytes directly
  instead of going through `Display`/`String` formatting (eliminates the
  `Display` + `String` allocation overhead; still uses one `Vec<u8>` alloc).
- `Debug` impl for `BaseEncoded` tightened (removed unused
  `Clone + Into<Vec<u8>>` bound).
- Added `impl AsRef<[u8]> for Varbytes` for future zero-alloc paths.
- Reconciled `serde` alloc feature vs std-only crate: changed `serde`
  dependency from `features = ["alloc"]` to `features = ["std"]` to match
  the crate's std-only reality. Updated `lib.rs` doc note from `no_std` Note
  to `std` Requirement.

### Tests
- Added 6 serde bounds-check tests: `test_varbytes_serde_len_exceeds_buffer_is_err_not_panic`,
  `test_varbytes_serde_len_exceeds_buffer_binary`,
  `test_varbytes_serde_len_exceeds_max_is_err`,
  `test_varbytes_serde_len_just_under_max_ok`,
  `test_varbytes_serde_len_just_over_max_is_err`,
  `test_varbytes_serde_valid_roundtrip`.

## [1.0.3] - 2026-07-15

### Added

- **`#![deny(unsafe_code)]`** at the crate root.
- **`#[must_use]`** on `Error::custom()`.
- **MSRV declared**: `rust-version = "1.85"` in `Cargo.toml`. CI verifies the
  MSRV with a dedicated job.
- **`cargo audit`** job in CI.
- **`cargo fmt --check`** and **`clippy -D warnings`** steps in CI.
- **Clippy lint configuration**: `[lints.clippy]` with `pedantic`, `nursery`,
  and `cargo` groups (all `warn`), plus `[lints.rust] unsafe_code = "deny"`.
- **`no_std` documentation** in the crate root noting that the crate is
  currently std-only and the `serde` alloc feature config does not make the
  crate `no_std`.
- **`Varuint` module documentation** explaining the relationship between
  `Varuint` and the integer trait impls in `multi-trait`.
- **`# Errors`** doc section on `BaseEncoder::from_base_encoded`.
- **`#[inline]`** on `Deref`/`DerefMut` impls (replaced `#[inline(always)]`).

### Changed

- **Edition 2024**: Updated from Rust 2021.
- **`Varbytes::encode_into`**: Replaced `v.append(&mut self.0.clone())` with
  `v.extend_from_slice(&self.0)` to avoid cloning the entire payload.
- **`Varbytes` serde `Serialize`**: Replaced
  `v.append(&mut self.as_bytes().to_vec())` with
  `v.extend_from_slice(self.as_bytes())` to avoid an intermediate allocation.
- **`DetectedEncoder::from_base_encoded`**: Bails on the first strict decode
  success instead of collecting all successful decodings, avoiding O(n)
  redundant decodes and false positives from overlapping alphabets.
- **`DetectedEncoder` doc comment**: Fixed typos and formatting.
- **Clippy pedantic/nursery/cargo warnings** resolved across all source.

## [1.0.2] - 2026-07-13

### Changed
- Updated `multi-base`, `multi-codec`, and `multi-trait` dependencies from
  path-based to published crates.io versions.
- Fixed codec name references after multicodec table sync.

## [1.0.1] - 2026-07-13

### Fixed
- Fixed `use` import formatting in `src/serde/de.rs`, `src/varbytes.rs`, and
  `src/varuint.rs` (cosmetic — `core::` imports reformatted).

## [1.0.0] - 2026-07-13

### Changed
- Synced from bettersign workspace (bs-multiutil 0.7.0)
- Renamed crate from `bs-multiutil` to `multi-util`
- Initial published release on crates.io as `multi-util`

[1.0.5]: https://github.com/cryptidtech/multi-util/compare/v1.0.4...v1.0.5
[1.0.4]: https://github.com/cryptidtech/multi-util/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/cryptidtech/multi-util/compare/v1.0.0...v1.0.3
[1.0.2]: https://github.com/cryptidtech/multi-util/releases/tag/v1.0.2
[1.0.1]: https://github.com/cryptidtech/multi-util/releases/tag/v1.0.1
[1.0.0]: https://github.com/cryptidtech/multi-util/releases/tag/v1.0.0
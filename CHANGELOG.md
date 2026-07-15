# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

## [1.0.0] - 2026-07-13

### Changed
- Synced from bettersign workspace (bs-multiutil 0.7.0)
- Renamed crate from `bs-multiutil` to `multi-util`
- Initial published release on crates.io as `multi-util`

[1.0.3]: https://github.com/cryptidtech/multi-util/compare/v1.0.0...v1.0.3
[1.0.0]: https://github.com/cryptidtech/multi-util/releases/tag/v1.0.0
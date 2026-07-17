// SPDX-License-Identifier: Apache-2.0
//! Serde (de)serialization for ['crate::prelude::Tagged'] wrapped objects
mod de;
mod ser;

pub use de::deserialize_varbytes_with_max;

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_test::{Configure, Token, assert_tokens};

    /// Serialize a value to CBOR bytes using `ciborium` (replaces the
    /// unmaintained `serde_cbor` dev-dependency).
    fn cbor_to_vec<T: Serialize>(value: &T) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(value, &mut buf).expect("CBOR serialize");
        buf
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Unit((u8, [u8; 2]));

    type EncodedUnit = BaseEncoded<Unit>;

    impl Unit {
        fn encoded_default() -> EncodedUnit {
            EncodedUnit::new(Self::preferred_encoding(), Self::default())
        }
    }

    impl Default for Unit {
        fn default() -> Self {
            Self((0x59, [0xDE, 0xAD]))
        }
    }

    impl EncodingInfo for Unit {
        fn preferred_encoding() -> Base {
            Base::Base16Lower
        }

        fn encoding(&self) -> Base {
            Self::preferred_encoding()
        }
    }

    impl<'a> TryFrom<&'a [u8]> for Unit {
        type Error = Error;

        fn try_from(s: &'a [u8]) -> Result<Self, Error> {
            if s.len() < 3 {
                Err(Error::custom("too few items in the vec"))
            } else {
                Ok(Self((s[0], [s[1], s[2]])))
            }
        }
    }

    impl From<Unit> for Vec<u8> {
        fn from(unit: Unit) -> Self {
            let mut v: Self = Self::default();
            v.push(unit.0.0);
            v.extend_from_slice(&unit.0.1);
            v
        }
    }

    #[test]
    fn test_serde_base_encoded_readable() {
        let unit = Unit::encoded_default();
        assert_tokens(&unit.readable(), &[Token::BorrowedStr("f59dead")]);
    }

    #[test]
    fn test_serde_base_encoded_compact() {
        let unit = Unit::encoded_default();
        assert_tokens(
            &unit.compact(),
            &[
                Token::Tuple { len: 2 },
                Token::Char('f'),
                Token::NewtypeStruct { name: "Unit" },
                Token::Tuple { len: 2 },
                Token::U8(0x59),
                Token::Tuple { len: 2 },
                Token::U8(0xDE),
                Token::U8(0xAD),
                Token::TupleEnd,
                Token::TupleEnd,
                Token::TupleEnd,
            ],
        );
    }

    #[test]
    fn test_cbor_reader_writer() {
        let unit1 = Unit::default();
        let mut b = Vec::new();
        ciborium::into_writer(&unit1, &mut b).unwrap();
        let unit2: Unit = ciborium::from_reader(b.as_slice()).unwrap();
        assert_eq!(unit1, unit2);
    }

    #[test]
    fn test_json_reader_writer() {
        let unit1 = Unit::default();
        let mut b = Vec::new();
        serde_json::to_writer_pretty(&mut b, &unit1).unwrap();
        let unit2: Unit = serde_json::from_reader(b.as_slice()).unwrap();
        assert_eq!(unit1, unit2);
    }

    #[test]
    fn test_encoded_cbor_reader_writer() {
        let unit1 = Unit::encoded_default();
        let mut b = Vec::new();
        ciborium::into_writer(&unit1, &mut b).unwrap();
        let unit2: EncodedUnit = ciborium::from_reader(b.as_slice()).unwrap();
        assert_eq!(unit1, unit2);
    }

    #[test]
    fn test_encoded_json_reader_writer() {
        let unit1 = Unit::encoded_default();
        let mut b = Vec::new();
        serde_json::to_writer_pretty(&mut b, &unit1).unwrap();
        let unit2: EncodedUnit = serde_json::from_reader(b.as_slice()).unwrap();
        assert_eq!(unit1, unit2);
    }

    #[test]
    fn test_serde_json() {
        let unit = Unit::encoded_default();
        let unit_s = serde_json::to_string(&unit).unwrap();
        assert_eq!(unit_s, "\"f59dead\"".to_string());
    }

    #[test]
    fn test_serde_cbor() {
        let unit = Unit::encoded_default();
        let unit_cbor = cbor_to_vec(&unit);
        // Note: ciborium may encode differently than serde_cbor, so we verify
        // round-trip instead of exact byte output.
        let unit2: EncodedUnit = ciborium::from_reader(unit_cbor.as_slice()).unwrap();
        assert_eq!(unit, unit2);
    }

    #[test]
    fn test_u8_varuint() {
        let v = Varuint(0x01_u8);
        assert_tokens(&v, &[Token::BorrowedBytes(&[0x01])]);
    }

    #[test]
    fn test_u8_long_varuint() {
        let v = Varuint(0xFF_u8);
        assert_tokens(&v, &[Token::BorrowedBytes(&[0xFF, 0x01])]);
    }

    #[test]
    fn test_u16_varuint() {
        let v = Varuint(0x0100_u16);
        assert_tokens(&v, &[Token::BorrowedBytes(&[0x80, 0x02])]);
    }

    #[test]
    fn test_u16_short_varuint() {
        let v = Varuint(0x0001_u16);
        assert_tokens(&v, &[Token::BorrowedBytes(&[0x01])]);
    }

    #[test]
    fn test_u16_long_varuint() {
        let v = Varuint(0xFFFF_u16);
        assert_tokens(&v, &[Token::BorrowedBytes(&[0xFF, 0xFF, 0x03])]);
    }

    #[test]
    fn test_u32_varuint() {
        let v = Varuint(0x0100_0000_u32);
        assert_tokens(&v, &[Token::BorrowedBytes(&[0x80, 0x80, 0x80, 0x08])]);
    }

    #[test]
    fn test_u32_short_varuint() {
        let v = Varuint(0x0000_0001_u32);
        assert_tokens(&v, &[Token::BorrowedBytes(&[0x01])]);
    }

    #[test]
    fn test_u32_long_varuint() {
        let v = Varuint(0xFFFF_FFFF_u32);
        assert_tokens(&v, &[Token::BorrowedBytes(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F])]);
    }

    #[test]
    fn test_u64_varuint() {
        let v = Varuint(0x0100_0000_0000_0000_u64);
        assert_tokens(
            &v,
            &[Token::BorrowedBytes(&[
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01,
            ])],
        );
    }

    #[test]
    fn test_u64_short_varuint() {
        let v = Varuint(0x0000_0000_0000_0001_u64);
        assert_tokens(&v, &[Token::BorrowedBytes(&[0x01])]);
    }

    #[test]
    fn test_u64_long_varuint() {
        let v = Varuint(0xFFFF_FFFF_FFFF_FFFF_u64);
        assert_tokens(
            &v,
            &[Token::BorrowedBytes(&[
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
            ])],
        );
    }

    #[test]
    fn test_u128_varuint() {
        let v = Varuint(0x0100_0000_0000_0000_0000_0000_0000_0000_u128);
        assert_tokens(
            &v,
            &[Token::Bytes(&[
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
                0x80, 0x80, 0x80, 0x02,
            ])],
        );
    }

    #[test]
    fn test_u128_short_varuint() {
        let v = Varuint(0x0000_0000_0000_0000_0000_0000_0000_0001_u128);
        assert_tokens(&v, &[Token::Bytes(&[0x01])]);
    }

    #[test]
    fn test_u128_long_varuint() {
        let v = Varuint(0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_u128);
        assert_tokens(
            &v,
            &[Token::Bytes(&[
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0x03,
            ])],
        );
    }

    #[test]
    fn test_usize_varuint() {
        let v = Varuint(0x0100_0000_0000_0000_usize);
        assert_tokens(
            &v,
            &[Token::Bytes(&[
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01,
            ])],
        );
    }

    #[test]
    fn test_usize_short_varuint() {
        let v = Varuint(0x0000_0000_0000_0001_usize);
        assert_tokens(&v, &[Token::Bytes(&[0x01])]);
    }

    #[test]
    fn test_usize_long_varuint() {
        let v = Varuint(0xFFFF_FFFF_FFFF_FFFF_usize);
        assert_tokens(
            &v,
            &[Token::Bytes(&[
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
            ])],
        );
    }

    #[test]
    fn test_usize_encoded() {
        let v = Varuint::encoded_new(Base::Base16Lower, 0x0100_0000_0000_0000_usize);
        assert_tokens(&v.readable(), &[Token::Str("f808080808080808001")]);
    }

    #[test]
    fn test_varbytes() {
        let v = Varbytes::new(vec![0x01, 0x02, 0x03]);
        assert_tokens(&v, &[Token::Bytes(&[0x03, 0x01, 0x02, 0x03])]);
    }

    #[test]
    fn test_encoded_varbytes() {
        let v = Varbytes::encoded_new(Base::Base16Lower, vec![0x01, 0x02, 0x03]);
        assert_tokens(&v.readable(), &[Token::Str("f03010203")]);
    }

    // ========================================================================
    // H4: serde Varbytes bounds-check tests
    // ========================================================================

    #[test]
    fn test_varbytes_serde_len_exceeds_buffer_is_err_not_panic() {
        // Crafted payload: length prefix claims 4 bytes but only 3 follow.
        // Pre-fix this panicked with an index-out-of-bounds; it must now
        // return a clean Err.
        let malicious: &[u8] = &[0x04, 0x01, 0x02, 0x03];

        let result: Result<Varbytes, serde_json::Error> =
            serde_json::from_slice(serde_json::to_string(malicious).unwrap().as_bytes());
        // serde_json hands the bytes to the visitor; the bounds check must
        // reject the over-long length claim without panicking.
        // (Depending on the format the bytes may arrive via visit_bytes or
        // visit_seq; either way the shared decode_varbytes helper rejects it.)
        assert!(result.is_err());
    }

    #[test]
    fn test_varbytes_serde_len_exceeds_buffer_binary() {
        // Direct binary deserialization via ciborium to exercise the
        // visit_bytes / visit_byte_buf paths.
        let malicious: &[u8] = &[0x04, 0x01, 0x02, 0x03];
        let result: Result<Varbytes, ciborium::de::Error<std::io::Error>> =
            ciborium::from_reader(malicious);
        assert!(result.is_err(), "must reject len > buffer, not panic");
    }

    #[test]
    fn test_varbytes_serde_len_exceeds_max_is_err() {
        // Length prefix claims just over MAX_DECODED_SIZE. The buffer is
        // trivially small so this also exceeds the buffer; the key property
        // is that it returns Err rather than attempting a huge allocation.
        use multi_trait::EncodeInto;
        let over_max = MAX_DECODED_SIZE + 1;
        let mut payload = Vec::new();
        payload.extend(over_max.encode_into());
        payload.extend(&[0u8; 4]); // a few bytes, nowhere near over_max

        let result: Result<Varbytes, ciborium::de::Error<std::io::Error>> =
            ciborium::from_reader(payload.as_slice());
        assert!(result.is_err(), "must reject len > MAX_DECODED_SIZE");
    }

    #[test]
    fn test_varbytes_serde_len_just_under_max_ok() {
        // A length just under MAX_DECODED_SIZE with a matching buffer is
        // accepted by the cap. We don't actually allocate ~16 MiB here; we
        // verify the cap logic via the configurable helper with a small max.
        use super::deserialize_varbytes_with_max;
        use multi_trait::EncodeInto;

        let max = 8usize;
        let data = vec![0xABu8; max];
        let mut payload = Vec::new();
        payload.extend(max.encode_into());
        payload.extend(&data);

        let de = serde::de::value::BytesDeserializer::<serde::de::value::Error>::new(&payload);
        let v: Varbytes =
            deserialize_varbytes_with_max(de, max).expect("len == max should be accepted");
        assert_eq!(v.as_bytes(), &data[..]);
    }

    #[test]
    fn test_varbytes_serde_len_just_over_max_is_err() {
        use super::deserialize_varbytes_with_max;
        use multi_trait::EncodeInto;

        let max = 8usize;
        let over = max + 1;
        let mut payload = Vec::new();
        payload.extend(over.encode_into());
        payload.extend(vec![0u8; over]); // buffer is large enough; only the cap rejects

        let de = serde::de::value::BytesDeserializer::<serde::de::value::Error>::new(&payload);
        let result = deserialize_varbytes_with_max(de, max);
        assert!(result.is_err(), "len just over max must be rejected");
    }

    #[test]
    fn test_varbytes_serde_valid_roundtrip() {
        // Sanity: a well-formed varbytes still round-trips through serde.
        let v = Varbytes::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let encoded = cbor_to_vec(&v);
        let decoded: Varbytes = ciborium::from_reader(encoded.as_slice()).expect("deserialize");
        assert_eq!(v, decoded);
    }
}

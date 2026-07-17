// SPDX-License-Identifier: Apache-2.0
use crate::{BaseEncoded, EncodingInfo, Error};
use core::{fmt, ops};
use multi_base::Base;
use multi_trait::prelude::{EncodeInto, TryDecodeFrom};

/// A wrapper type to handle serde of byte arrays as bytes
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Varbytes(Vec<u8>);

/// Maximum number of bytes a single [`Varbytes`] value will allocate when
/// decoded from untrusted wire data.
///
/// The 16 MiB ceiling comfortably exceeds every legitimate multiformat payload
/// handled by this crate stack (the largest is a Classic `McEliece` secret key
/// at a few hundred KiB) while bounding the worst-case allocation an attacker
/// can trigger with a crafted length prefix. Callers that need a different
/// bound should validate the raw buffer length before invoking
/// [`Varbytes::try_decode_from`].
pub const MAX_DECODED_SIZE: usize = 16 * 1024 * 1024;

/// type alias for a Varbytes base encoded to/from string
pub type EncodedVarbytes = BaseEncoded<Varbytes>;

impl Varbytes {
    /// Create a new Varbytes from a `Vec<u8>`
    #[must_use]
    pub const fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// create an encoded varbytes
    #[must_use]
    pub const fn encoded_new(base: Base, v: Vec<u8>) -> EncodedVarbytes {
        BaseEncoded::new(base, Self::new(v))
    }

    /// Get a reference to the inner byte slice
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Get a mutable reference to the inner byte vector
    pub const fn as_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }

    /// consume self and return inner vec
    #[must_use]
    pub fn to_inner(self) -> Vec<u8> {
        self.0
    }
}

impl fmt::Debug for Varbytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.encode_into().as_slice())
    }
}

impl ops::Deref for Varbytes {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Varbytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl EncodingInfo for Varbytes {
    fn preferred_encoding() -> Base {
        Base::Base16Lower
    }

    fn encoding(&self) -> Base {
        Base::Base16Lower
    }
}

impl From<Varbytes> for Vec<u8> {
    fn from(vb: Varbytes) -> Self {
        vb.encode_into()
    }
}

impl EncodeInto for Varbytes {
    fn encode_into(&self) -> Vec<u8> {
        let mut v = self.0.len().encode_into();
        v.extend_from_slice(&self.0);
        v
    }
}

impl<'a> TryFrom<&'a [u8]> for Varbytes {
    type Error = Error;

    fn try_from(s: &'a [u8]) -> Result<Self, Error> {
        let (v, _) = Self::try_decode_from(s)?;
        Ok(v)
    }
}

impl<'a> TryDecodeFrom<'a> for Varbytes {
    type Error = Error;

    fn try_decode_from(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Self::Error> {
        let (len, ptr) = usize::try_decode_from(bytes)?;

        // Reject length claims that exceed the configured maximum decoded size.
        // This bounds the worst-case allocation for untrusted wire data and
        // mitigates CWE-400 (Uncontrolled Resource Consumption).
        if len > MAX_DECODED_SIZE {
            return Err(Error::InputTooLarge {
                claimed: len,
                max: MAX_DECODED_SIZE,
            });
        }

        // Validate buffer has enough data for claimed length
        // This prevents buffer overflow (CWE-125) when length claim exceeds available data
        if len > ptr.len() {
            return Err(Error::InsufficientData {
                expected: len,
                actual: ptr.len(),
            });
        }

        let v = ptr[..len].to_vec();
        let ptr = &ptr[len..];
        Ok((Self(v), ptr))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let v = Varbytes::default();
        assert_eq!(Vec::<u8>::default(), *v);
    }

    #[test]
    fn test_to_inner() {
        let v = Varbytes::new(vec![1, 2, 3]);
        assert_eq!(vec![1, 2, 3], v.to_inner());
    }

    #[test]
    fn test_default_round_trip() {
        let v1 = Varbytes::default();
        let v: Vec<u8> = v1.clone().into();
        let v2 = Varbytes::try_from(v.as_slice()).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_encode_decode_round_trip() {
        let v1 = Varbytes::new(vec![1, 2, 3]);
        let (v2, _) = Varbytes::try_decode_from(&v1.encode_into()).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_into_tryfrom_round_trip() {
        let v1 = Varbytes::new(vec![1, 2, 3]);
        let data: Vec<u8> = v1.clone().into();
        let v2 = Varbytes::try_from(data.as_slice()).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_debug() {
        let v = Varbytes::new(vec![1, 2, 3]);
        assert_eq!("[3, 1, 2, 3]".to_string(), format!("{v:?}"));
    }

    // ============================================================================
    // SECURITY TESTS - CRIT-1: Buffer Overflow Prevention
    // ============================================================================

    #[test]
    fn test_crit1_buffer_overflow_prevented() {
        // CRIT-1: Test that buffer overflow vulnerability is fixed
        //
        // Attack scenario: Attacker crafts input with large length claim
        // but provides minimal actual data, attempting to trigger out-of-bounds read

        use multi_trait::EncodeInto;

        // Create malicious input: claims 4GB length but only has 3 bytes
        let mut malicious = Vec::new();

        // Encode a huge length (0xFFFFFFFF = ~4GB)
        let huge_length = 0xFFFF_FFFF_usize;
        malicious.extend(huge_length.encode_into());

        // Provide only 3 bytes of actual data
        malicious.extend(&[0x01, 0x02, 0x03]);

        // Attempt to decode - should fail, not panic
        let result = Varbytes::try_decode_from(&malicious);

        assert!(
            result.is_err(),
            "Should reject length claim that exceeds available data"
        );

        // Verify correct error type. The 4GB claim exceeds both the
        // MAX_DECODED_SIZE cap (16 MiB) and the available buffer (3 bytes);
        // either InputTooLarge or InsufficientData is an acceptable rejection
        // — both prevent the out-of-bounds read.
        match result.unwrap_err() {
            Error::InputTooLarge { claimed, max } => {
                assert_eq!(claimed, huge_length);
                assert_eq!(max, MAX_DECODED_SIZE);
            }
            Error::InsufficientData { expected, actual } => {
                assert_eq!(expected, huge_length);
                assert_eq!(actual, 3);
            }
            e => panic!("Expected InputTooLarge or InsufficientData error, got: {e:?}"),
        }
    }

    #[test]
    fn test_crit1_regression_zero_length() {
        // Regression test: zero-length varbytes should work
        use multi_trait::EncodeInto;

        let encoded = 0usize.encode_into();
        let result = Varbytes::try_decode_from(&encoded);

        assert!(result.is_ok(), "Zero-length varbytes should be valid");

        let (varbytes, remaining) = result.unwrap();
        assert_eq!(varbytes.to_inner(), Vec::<u8>::new());
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_crit1_regression_exact_length() {
        // Regression test: exact length match should work
        use multi_trait::EncodeInto;

        let data = vec![0xAA, 0xBB, 0xCC];
        let mut encoded = data.len().encode_into();
        encoded.extend(&data);

        let result = Varbytes::try_decode_from(&encoded);

        assert!(result.is_ok(), "Exact length match should succeed");

        let (varbytes, remaining) = result.unwrap();
        assert_eq!(varbytes.to_inner(), data);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_crit1_one_byte_over() {
        // Boundary test: length claim exceeds buffer by 1 byte
        use multi_trait::EncodeInto;

        let mut malicious = Vec::new();

        // Claim 4 bytes
        malicious.extend(4usize.encode_into());

        // Provide only 3 bytes
        malicious.extend(&[0x01, 0x02, 0x03]);

        let result = Varbytes::try_decode_from(&malicious);

        assert!(
            result.is_err(),
            "Should reject length claim exceeding buffer by 1"
        );

        match result.unwrap_err() {
            Error::InsufficientData { expected, actual } => {
                assert_eq!(expected, 4);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_crit1_empty_buffer_nonzero_length() {
        // Edge case: length > 0 but no data provided
        use multi_trait::EncodeInto;

        let mut malicious = Vec::new();

        // Claim 100 bytes
        malicious.extend(100usize.encode_into());

        // Provide no data
        // (nothing appended)

        let result = Varbytes::try_decode_from(&malicious);

        assert!(result.is_err(), "Should reject nonzero length with no data");
    }

    #[test]
    fn test_crit1_legitimate_large_data() {
        // Verify legitimate large data still works
        use multi_trait::EncodeInto;

        // Create 1MB of legitimate data
        let large_data = vec![0x42; 1024 * 1024];
        let mut encoded = large_data.len().encode_into();
        encoded.extend(&large_data);

        let result = Varbytes::try_decode_from(&encoded);

        assert!(result.is_ok(), "Legitimate large data should be accepted");

        let (varbytes, remaining) = result.unwrap();
        let inner = varbytes.to_inner();
        assert_eq!(inner.len(), 1024 * 1024);
        assert_eq!(inner[0], 0x42);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_crit1_with_trailing_data() {
        // Test that remaining bytes are correctly returned
        use multi_trait::EncodeInto;

        let data = vec![0xAA, 0xBB];
        let trailing = vec![0xCC, 0xDD, 0xEE];

        let mut encoded = data.len().encode_into();
        encoded.extend(&data);
        encoded.extend(&trailing);

        let result = Varbytes::try_decode_from(&encoded);

        assert!(result.is_ok());

        let (varbytes, remaining) = result.unwrap();
        assert_eq!(varbytes.to_inner(), data);
        assert_eq!(remaining, trailing.as_slice());
    }

    // ============================================================================
    // PROPERTY-BASED TESTS - CRIT-1
    // ============================================================================

    #[cfg(test)]
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_crit1_never_panics_on_random_input(
            bytes in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            // Most important property: should never panic on any input
            let _ = Varbytes::try_decode_from(&bytes);
            // Should either succeed or return error, never panic
        }

        #[test]
        fn prop_crit1_valid_roundtrip(
            data in prop::collection::vec(any::<u8>(), 0..10000)
        ) {
            // Valid varbytes should roundtrip correctly
            let varbytes = Varbytes::new(data.clone());
            let encoded = varbytes.encode_into();
            let result = Varbytes::try_decode_from(&encoded);

            prop_assert!(result.is_ok());

            let (decoded, remaining) = result.unwrap();
            prop_assert_eq!(decoded.to_inner(), data);
            prop_assert!(remaining.is_empty());
        }

        #[test]
        fn prop_crit1_length_mismatch_detected(
            claimed_len in 1usize..1000,
            actual_len in 0usize..100
        ) {
            use multi_trait::EncodeInto;

            // Only test cases where claim exceeds actual
            if claimed_len > actual_len {
                let mut malicious = Vec::new();
                malicious.extend(claimed_len.encode_into());
                malicious.extend(vec![0u8; actual_len]);

                let result = Varbytes::try_decode_from(&malicious);

                // Should always reject length claim > available data
                prop_assert!(result.is_err());

                match result.unwrap_err() {
                    Error::InsufficientData { expected, actual } => {
                        prop_assert_eq!(expected, claimed_len);
                        prop_assert_eq!(actual, actual_len);
                    }
                    e => return Err(TestCaseError::fail(format!("Wrong error type: {e:?}"))),
                }
            }
        }
    }
}

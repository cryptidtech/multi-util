// SPDX-License-Identifier: Apache-2.0
use crate::{BaseEncoded, BaseEncoder, EncodingInfo, Varbytes, Varuint, varbytes::VarbytesMax};
#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{fmt, marker};
use multi_base::Base;
use multi_trait::prelude::TryDecodeFrom;
use serde::de;

/// Deserialize instance of [`crate::BaseEncoded`] from a byte slice
impl<'de, T, Enc> de::Deserialize<'de> for BaseEncoded<T, Enc>
where
    T: de::Deserialize<'de> + EncodingInfo + for<'a> TryFrom<&'a [u8]>,
    Enc: BaseEncoder,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Clone, Default)]
        struct BaseEncodedVisitor<T, Enc> {
            _enc: marker::PhantomData<Enc>,
            _t: marker::PhantomData<T>,
        }

        impl<'de, T, Enc> de::Visitor<'de> for BaseEncodedVisitor<T, Enc>
        where
            T: de::Deserialize<'de> + EncodingInfo + for<'a> TryFrom<&'a [u8]>,
            Enc: BaseEncoder,
        {
            type Value = BaseEncoded<T, Enc>;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "borrowed str, str, String, or tuple of (u8, T)")
            }

            // human readable

            // shortest lifetime
            #[inline]
            fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::Value::try_from(s).map_err(|e| de::Error::custom(e.to_string()))
            }

            #[inline]
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::Value::try_from(s).map_err(|e| de::Error::custom(e.to_string()))
            }

            // longest lifetime
            #[inline]
            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::Value::try_from(s.as_str()).map_err(|e| de::Error::custom(e.to_string()))
            }

            // binary
            #[inline]
            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: de::SeqAccess<'de>,
            {
                let base = match seq.next_element::<char>()? {
                    Some(b) => Base::from_code(b).map_err(|e| de::Error::custom(e.to_string()))?,
                    None => {
                        return Err(de::Error::custom("expected base encoding char".to_string()));
                    }
                };

                let t = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("expected inner type value".to_string()))?;

                Ok(Self::Value {
                    enc: marker::PhantomData,
                    base,
                    t,
                })
            }
        }

        deserializer.deserialize_any(BaseEncodedVisitor {
            _enc: marker::PhantomData,
            _t: marker::PhantomData,
        })
    }
}

/// Deserialize instance of [`crate::Varuint`] from a byte slice
impl<'de, T> de::Deserialize<'de> for Varuint<T>
where
    T: for<'a> TryDecodeFrom<'a>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct VaruintVisitor<T>(marker::PhantomData<T>);

        impl<'de, T> de::Visitor<'de> for VaruintVisitor<T>
        where
            T: for<'a> TryDecodeFrom<'a>,
        {
            type Value = Varuint<T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "varuint encoded numeric value")
            }

            // only binary

            // shortest lifetime
            #[inline]
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (t, _) = T::try_decode_from(v)
                    .map_err(|_| de::Error::custom("failed to deserialize varuint bytes"))?;
                Ok(Varuint(t))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (t, _) = T::try_decode_from(v)
                    .map_err(|_| de::Error::custom("failed to deserialize varuint bytes"))?;
                Ok(Varuint(t))
            }

            // longest lifetime
            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (t, _) = T::try_decode_from(v.as_slice())
                    .map_err(|_| de::Error::custom("failed to deserialize varuint bytes"))?;
                Ok(Varuint(t))
            }

            // binary / human readable

            // this typically only happens when there are bytes serialized into
            // a human readable format.
            #[inline]
            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: de::SeqAccess<'de>,
            {
                let mut v = Vec::new();
                while let Some(b) = seq.next_element()? {
                    v.push(b);
                }
                let (t, _) = T::try_decode_from(v.as_slice())
                    .map_err(|_| de::Error::custom("failed to deserialize varuint bytes"))?;
                Ok(Varuint(t))
            }
        }

        deserializer.deserialize_bytes(VaruintVisitor::<T>(marker::PhantomData::<T>))
    }
}

/// Deserialize instance of [`crate::VarbytesMax`] from a byte slice.
///
/// The `MAX` const generic sets the maximum decoded size. The default
/// (`DEFAULT_MAX = 16 MiB`) applies when deserializing the [`Varbytes`] alias.
/// Use `VarbytesMax<N>` directly for a custom cap at the type level. For a
/// per-field override without a distinct type, use
/// [`deserialize_varbytes_with_max`].
impl<'de, const MAX: usize> de::Deserialize<'de> for VarbytesMax<MAX> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct VarbytesVisitorMaxConst<const MAX: usize>;

        impl<'de, const M: usize> de::Visitor<'de> for VarbytesVisitorMaxConst<M> {
            type Value = VarbytesMax<M>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "varuint encoded len followed by bytes")
            }

            // only binary

            // shortest lifetime
            #[inline]
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                decode_varbytes(v, M).map(VarbytesMax::new)
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                decode_varbytes(v, M).map(VarbytesMax::new)
            }

            // longest lifetime
            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                decode_varbytes(v.as_slice(), M).map(VarbytesMax::new)
            }

            // binary / human readable

            // This typically only happens when bytes are serialized into a
            // human-readable format.
            #[inline]
            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: de::SeqAccess<'de>,
            {
                let mut v = Vec::new();
                while let Some(b) = seq.next_element()? {
                    v.push(b);
                }
                decode_varbytes(v.as_slice(), M).map(VarbytesMax::new)
            }
        }

        deserializer.deserialize_bytes(VarbytesVisitorMaxConst::<MAX>)
    }
}

/// Decode a `Varbytes` payload from a byte slice with a caller-supplied
/// maximum decoded size.
///
/// This is the shared bounds-checked decode path used by the
/// [`VarbytesMax`] [`Deserialize`](de::Deserialize) impl and by
/// [`deserialize_varbytes_with_max`]. It mirrors the safety checks in
/// [`VarbytesMax::try_decode_from`](crate::VarbytesMax::try_decode_from).
/// After decoding the length prefix, it rejects lengths that exceed the
/// available buffer (preventing an out-of-bounds read or panic) and lengths
/// that exceed the supplied maximum (preventing unbounded allocation). It
/// returns the raw decoded bytes. The caller wraps them in the appropriate
/// `VarbytesMax<MAX>` type.
fn decode_varbytes<E>(input: &[u8], max: usize) -> Result<Vec<u8>, E>
where
    E: de::Error,
{
    let (len, ptr) = usize::try_decode_from(input)
        .map_err(|_| de::Error::custom("failed to deserialize varuint len"))?;

    if len > max {
        return Err(de::Error::custom("varbytes length exceeds maximum"));
    }
    if len > ptr.len() {
        return Err(de::Error::custom("varbytes length exceeds buffer"));
    }

    let v = ptr[..len].to_vec();
    Ok(v)
}

/// Deserialize a [`Varbytes`] with a caller-specified maximum decoded size,
/// overriding the default [`DEFAULT_MAX`] cap.
///
/// Use this with `#[serde(deserialize_with = "...")]` when a field needs a
/// tighter or looser bound than the crate-wide default:
///
/// ```
/// # use serde::Deserialize;
/// # use multi_util::Varbytes;
/// # use multi_util::serde::deserialize_varbytes_with_max;
/// #[derive(Deserialize)]
/// struct Small {
///     #[serde(deserialize_with = "deserialize_varbytes_with_max_256")]
///     data: Varbytes,
/// }
///
/// // Wrapper that fixes the max at 256 bytes for use with
/// // `#[serde(deserialize_with = "...")]` (which requires an `fn(D) -> Result<T, D::Error>`).
/// fn deserialize_varbytes_with_max_256<'de, D>(deserializer: D) -> Result<Varbytes, D::Error>
/// where
///     D: serde::Deserializer<'de>,
/// {
///     deserialize_varbytes_with_max(deserializer, 256)
/// }
/// ```
///
/// The `max` argument is the maximum number of bytes the decoded `Varbytes`
/// value may contain. A length prefix claiming more than `max` bytes (or more
/// bytes than remain in the buffer) is rejected with a clean `Err`, never a
/// panic.
///
/// # Errors
///
/// Returns a deserializer error if:
/// - the length prefix cannot be decoded as a varuint,
/// - the decoded length exceeds `max`,
/// - the decoded length exceeds the remaining buffer.
pub fn deserialize_varbytes_with_max<'de, D>(
    deserializer: D,
    max: usize,
) -> Result<Varbytes, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct VarbytesVisitorMax(usize);

    impl<'de> de::Visitor<'de> for VarbytesVisitorMax {
        type Value = Varbytes;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "varuint encoded len followed by bytes")
        }

        #[inline]
        fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            decode_varbytes(v, self.0).map(Varbytes::new)
        }

        #[inline]
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            decode_varbytes(v, self.0).map(Varbytes::new)
        }

        #[inline]
        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            decode_varbytes(v.as_slice(), self.0).map(Varbytes::new)
        }

        #[inline]
        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            let mut v = Vec::new();
            while let Some(b) = seq.next_element()? {
                v.push(b);
            }
            decode_varbytes(v.as_slice(), self.0).map(Varbytes::new)
        }
    }

    deserializer.deserialize_bytes(VarbytesVisitorMax(max))
}

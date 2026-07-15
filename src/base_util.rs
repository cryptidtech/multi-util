// SPDX-License-Identifier: Apache-2.0
use crate::prelude::Base;

/// convert a multibase Base to its string equivalent
#[must_use]
pub fn base_name(b: Base) -> String {
    use Base::{
        Base2, Base8, Base10, Base16Lower, Base16Upper, Base32HexLower, Base32HexPadLower,
        Base32HexPadUpper, Base32HexUpper, Base32Lower, Base32PadLower, Base32PadUpper,
        Base32Upper, Base32Z, Base36Lower, Base36Upper, Base58Btc, Base58Flickr, Base64, Base64Pad,
        Base64Url, Base64UrlPad, Base256Emoji, Identity,
    };
    match b {
        Identity => "Identity",
        Base2 => "Base2",
        Base8 => "Base8",
        Base10 => "Base10",
        Base16Lower => "Base16Lower",
        Base16Upper => "Base16Upper",
        Base32Lower => "Base32Lower",
        Base32Upper => "Base32Upper",
        Base32PadLower => "Base32PadLower",
        Base32PadUpper => "Base32PadUpper",
        Base32HexLower => "Base32HexLower",
        Base32HexUpper => "Base32HexUpper",
        Base32HexPadLower => "Base32HexPadLower",
        Base32HexPadUpper => "Base32HexPadUpper",
        Base32Z => "Base32Z",
        Base36Lower => "Base36Lower",
        Base36Upper => "Base36Upper",
        Base58Flickr => "Base58Flickr",
        Base58Btc => "Base58Btc",
        Base64 => "Base64",
        Base64Pad => "Base64Pad",
        Base64Url => "Base64Url",
        Base64UrlPad => "Base64UrlPad",
        Base256Emoji => "Base256Emoji",
    }
    .to_string()
}

/// Iterator over the Base enum values
pub struct BaseIter(Option<Base>);

impl Default for BaseIter {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseIter {
    /// create a new `BaseIter`
    #[must_use]
    pub const fn new() -> Self {
        Self(None)
    }
}

impl From<Base> for BaseIter {
    fn from(b: Base) -> Self {
        Self(Some(b))
    }
}

impl Iterator for BaseIter {
    type Item = Base;

    fn next(&mut self) -> Option<Self::Item> {
        use Base::{
            Base2, Base8, Base10, Base16Lower, Base16Upper, Base32HexLower, Base32HexPadLower,
            Base32HexPadUpper, Base32HexUpper, Base32Lower, Base32PadLower, Base32PadUpper,
            Base32Upper, Base32Z, Base36Lower, Base36Upper, Base58Btc, Base58Flickr, Base64,
            Base64Pad, Base64Url, Base64UrlPad, Base256Emoji, Identity,
        };
        let result = match self.0 {
            None => Identity,
            Some(b) => match b {
                Identity => Base2,
                Base2 => Base8,
                Base8 => Base10,
                Base10 => Base16Lower,
                Base16Lower => Base16Upper,
                Base16Upper => Base32Lower,
                Base32Lower => Base32Upper,
                Base32Upper => Base32PadLower,
                Base32PadLower => Base32PadUpper,
                Base32PadUpper => Base32HexLower,
                Base32HexLower => Base32HexUpper,
                Base32HexUpper => Base32HexPadLower,
                Base32HexPadLower => Base32HexPadUpper,
                Base32HexPadUpper => Base32Z,
                Base32Z => Base36Lower,
                Base36Lower => Base36Upper,
                Base36Upper => Base58Flickr,
                Base58Flickr => Base58Btc,
                Base58Btc => Base64,
                Base64 => Base64Pad,
                Base64Pad => Base64Url,
                Base64Url => Base64UrlPad,
                Base64UrlPad => Base256Emoji,
                Base256Emoji => return None,
            },
        };
        self.0 = Some(result);
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_iter() {
        assert_eq!(BaseIter::new().next(), Some(Base::Identity));
    }

    #[test]
    fn test_last_iter() {
        let mut iter: BaseIter = Base::Base256Emoji.into();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_all_iter() {
        let iter = BaseIter::new();
        for b in iter {
            println!("{}", base_name(b));
        }
    }
}

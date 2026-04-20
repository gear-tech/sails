//! Codec availability helpers for IDL annotations.
//!
//! Methods may carry a `@codec` annotation to restrict which dispatch paths
//! they participate in. The value is a comma-separated list of codecs
//! (e.g. `@codec: scale`, `@codec: ethabi`, `@codec: scale,ethabi`).
//! No codec annotations means both codecs (default).

use alloc::collections::BTreeSet;
use alloc::string::String;

type Annotation = (String, Option<String>);

/// Available codecs for a method's dispatch path.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Codec {
    Scale,
    Ethabi,
}

/// Resolves the set of codecs a method is available through based on its
/// `@codec` annotations.
///
/// Semantics:
/// - no `@codec` annotations → both codecs enabled (default)
/// - `@codec` with no value → both codecs enabled
/// - repeated `@codec` annotations → union of all declared codec values
/// - unknown codec tokens → ignored
/// - empty / whitespace-only tokens → ignored
pub fn codecs(annotations: &[Annotation]) -> BTreeSet<Codec> {
    let mut result = BTreeSet::new();
    let mut saw_codec_annotation = false;

    for (name, value) in annotations {
        if name != "codec" {
            continue;
        }
        saw_codec_annotation = true;
        match value {
            None => {
                result.insert(Codec::Scale);
                result.insert(Codec::Ethabi);
            }
            Some(value) => {
                for token in value
                    .split(',')
                    .map(str::trim)
                    .filter(|token| !token.is_empty())
                {
                    match token {
                        "scale" => {
                            result.insert(Codec::Scale);
                        }
                        "ethabi" => {
                            result.insert(Codec::Ethabi);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    if !saw_codec_annotation {
        result.insert(Codec::Scale);
        result.insert(Codec::Ethabi);
    }

    result
}

/// Returns `true` if the method is available through SCALE/Gear dispatch.
pub fn has_scale_codec(annotations: &[Annotation]) -> bool {
    codecs(annotations).contains(&Codec::Scale)
}

/// Returns `true` if the method is available through Solidity ABI dispatch.
pub fn has_ethabi_codec(annotations: &[Annotation]) -> bool {
    codecs(annotations).contains(&Codec::Ethabi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString as _;
    use alloc::vec;
    use alloc::vec::Vec;

    fn ann(name: &str, value: Option<&str>) -> Annotation {
        (name.to_string(), value.map(|s| s.to_string()))
    }

    #[test]
    fn no_annotations_means_both() {
        let anns: Vec<Annotation> = vec![];
        assert!(has_scale_codec(&anns));
        assert!(has_ethabi_codec(&anns));
    }

    #[test]
    fn codec_scale_only() {
        let anns = vec![ann("codec", Some("scale"))];
        assert!(has_scale_codec(&anns));
        assert!(!has_ethabi_codec(&anns));
    }

    #[test]
    fn codec_ethabi_only() {
        let anns = vec![ann("codec", Some("ethabi"))];
        assert!(!has_scale_codec(&anns));
        assert!(has_ethabi_codec(&anns));
    }

    #[test]
    fn codec_both_explicit() {
        let anns = vec![ann("codec", Some("scale,ethabi"))];
        assert!(has_scale_codec(&anns));
        assert!(has_ethabi_codec(&anns));
    }

    #[test]
    fn codec_both_with_spaces() {
        let anns = vec![ann("codec", Some("scale, ethabi"))];
        assert!(has_scale_codec(&anns));
        assert!(has_ethabi_codec(&anns));
    }

    #[test]
    fn codec_no_value_means_both() {
        let anns = vec![ann("codec", None)];
        assert!(has_scale_codec(&anns));
        assert!(has_ethabi_codec(&anns));
    }

    #[test]
    fn repeated_codec_annotations_are_merged() {
        let anns = vec![ann("codec", Some("scale")), ann("codec", Some("ethabi"))];
        assert!(has_scale_codec(&anns));
        assert!(has_ethabi_codec(&anns));
    }

    #[test]
    fn unknown_and_empty_tokens_are_ignored() {
        let anns = vec![ann("codec", Some("scale, ,unknown"))];
        assert!(has_scale_codec(&anns));
        assert!(!has_ethabi_codec(&anns));
    }

    #[test]
    fn other_annotations_are_ignored_when_codec_missing() {
        let anns = vec![ann("query", None), ann("payable", None)];
        assert!(has_scale_codec(&anns));
        assert!(has_ethabi_codec(&anns));
    }
}

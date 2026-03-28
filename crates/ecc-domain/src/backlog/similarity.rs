//! Duplicate detection — Levenshtein distance, keyword Jaccard, composite scoring.

use serde::Serialize;
use std::collections::HashSet;

/// Weight for Levenshtein similarity in composite score.
pub const LEVENSHTEIN_WEIGHT: f64 = 0.7;
/// Weight for keyword Jaccard index in composite score.
pub const JACCARD_WEIGHT: f64 = 0.3;
/// Score boost per matching tag.
pub const TAG_BOOST_PER_MATCH: f64 = 0.15;
/// Maximum tag boost (caps the number of matching tags that matter).
pub const TAG_BOOST_CAP: f64 = 0.3;
/// Minimum composite score to consider an entry a duplicate candidate.
pub const DUPLICATE_THRESHOLD: f64 = 0.6;

/// Maximum possible raw score before normalization.
const MAX_RAW_SCORE: f64 = LEVENSHTEIN_WEIGHT + JACCARD_WEIGHT + TAG_BOOST_CAP;

/// A candidate duplicate entry with similarity score.
#[derive(Debug, Clone, Serialize)]
pub struct DuplicateCandidate {
    pub id: String,
    pub title: String,
    pub score: f64,
}

/// Compute Levenshtein edit distance between two strings (Unicode-aware).
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

/// Normalized Levenshtein similarity: 1.0 = identical, 0.0 = completely different.
///
/// Operates on Unicode characters, not bytes.
pub fn normalized_levenshtein_similarity(a: &str, b: &str) -> f64 {
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    let dist = levenshtein_distance(a, b);
    1.0 - (dist as f64 / max_len as f64)
}

/// Split a title into lowercase keyword set.
fn keywords(title: &str) -> HashSet<String> {
    title
        .to_lowercase()
        .split(|c: char| c.is_whitespace() || c == '-' || c == '—' || c == '_')
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

/// Jaccard index of keyword sets: |intersection| / |union|.
pub fn keyword_jaccard(a: &str, b: &str) -> f64 {
    let set_a = keywords(a);
    let set_b = keywords(b);
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

/// Composite similarity score combining Levenshtein, Jaccard, and tag boost.
///
/// Returns a normalized score in [0.0, 1.0].
///
/// Raw formula: `LEVENSHTEIN_WEIGHT * normalized_lev + JACCARD_WEIGHT * jaccard + tag_boost`
/// Normalized by dividing by `MAX_RAW_SCORE` (1.0 + TAG_BOOST_CAP = 1.3).
pub fn composite_score(
    query_title: &str,
    entry_title: &str,
    query_tags: &[String],
    entry_tags: &[String],
) -> f64 {
    let lev =
        normalized_levenshtein_similarity(&query_title.to_lowercase(), &entry_title.to_lowercase());
    let jac = keyword_jaccard(query_title, entry_title);

    let query_tag_set: HashSet<&str> = query_tags.iter().map(|s| s.as_str()).collect();
    let matching_tags = entry_tags
        .iter()
        .filter(|t| query_tag_set.contains(t.as_str()))
        .count();
    let tag_boost = (TAG_BOOST_PER_MATCH * matching_tags as f64).min(TAG_BOOST_CAP);

    let raw = LEVENSHTEIN_WEIGHT * lev + JACCARD_WEIGHT * jac + tag_boost;
    (raw / MAX_RAW_SCORE).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_known_pair() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn levenshtein_empty_string() {
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
    }

    #[test]
    fn levenshtein_unicode_chars() {
        // em-dash "—" is 3 bytes but 1 char
        assert_eq!(levenshtein_distance("—", "x"), 1);
        // Two identical multi-byte strings
        assert_eq!(levenshtein_distance("café", "café"), 0);
        // One char difference in multi-byte string
        assert_eq!(levenshtein_distance("café", "cafè"), 1);
    }

    #[test]
    fn normalized_levenshtein_identical() {
        let sim = normalized_levenshtein_similarity("hello", "hello");
        assert!((sim - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn normalized_levenshtein_unicode_identical() {
        let sim = normalized_levenshtein_similarity("日本語テスト", "日本語テスト");
        assert!((sim - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn keyword_jaccard_partial_overlap() {
        let jac = keyword_jaccard(
            "Replace hooks with Rust",
            "Replace hooks with compiled Rust",
        );
        assert!((jac - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn keyword_jaccard_no_overlap() {
        let jac = keyword_jaccard("alpha beta", "gamma delta");
        assert!((jac - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn constants_are_defined() {
        assert!((LEVENSHTEIN_WEIGHT - 0.7).abs() < f64::EPSILON);
        assert!((JACCARD_WEIGHT - 0.3).abs() < f64::EPSILON);
        assert!((TAG_BOOST_PER_MATCH - 0.15).abs() < f64::EPSILON);
        assert!((TAG_BOOST_CAP - 0.3).abs() < f64::EPSILON);
        assert!((DUPLICATE_THRESHOLD - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn composite_score_tag_boost_capped() {
        let score = composite_score(
            "test",
            "test",
            &["a".into(), "b".into(), "c".into(), "d".into()],
            &["a".into(), "b".into(), "c".into(), "d".into()],
        );
        // Normalized: (0.7*1.0 + 0.3*1.0 + 0.3) / 1.3 = 1.3/1.3 = 1.0
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn composite_score_normalized_range() {
        // Max possible score should be 1.0
        let max_score = composite_score(
            "exact",
            "exact",
            &["a".into(), "b".into(), "c".into()],
            &["a".into(), "b".into(), "c".into()],
        );
        assert!(max_score <= 1.0, "max score {max_score} exceeds 1.0");

        // Min possible score should be >= 0.0
        let min_score = composite_score("aaaa", "zzzz", &[], &[]);
        assert!(min_score >= 0.0, "min score {min_score} below 0.0");
    }

    #[test]
    fn composite_score_similar_titles() {
        let score = composite_score(
            "Replace hooks with compiled Rust",
            "Replace hooks with Rust binaries",
            &["rust".into(), "hooks".into()],
            &["rust".into(), "hooks".into()],
        );
        assert!(
            score >= DUPLICATE_THRESHOLD,
            "expected >= {DUPLICATE_THRESHOLD}, got {score}"
        );
    }
}

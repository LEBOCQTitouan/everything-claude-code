//! Duplicate detection — Levenshtein distance, keyword Jaccard, composite scoring.

use serde::Serialize;
use std::collections::HashSet;

/// A candidate duplicate entry with similarity score.
#[derive(Debug, Clone, Serialize)]
pub struct DuplicateCandidate {
    pub id: String,
    pub title: String,
    pub score: f64,
}

/// Compute Levenshtein edit distance between two strings.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

/// Normalized Levenshtein similarity: 1.0 = identical, 0.0 = completely different.
pub fn normalized_levenshtein_similarity(a: &str, b: &str) -> f64 {
    let max_len = a.len().max(b.len());
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
/// Formula: `0.7 * normalized_levenshtein + 0.3 * keyword_jaccard + tag_boost`
/// where `tag_boost = min(0.3, 0.15 * matching_tag_count)`.
pub fn composite_score(
    query_title: &str,
    entry_title: &str,
    query_tags: &[String],
    entry_tags: &[String],
) -> f64 {
    let lev = normalized_levenshtein_similarity(
        &query_title.to_lowercase(),
        &entry_title.to_lowercase(),
    );
    let jac = keyword_jaccard(query_title, entry_title);

    let query_tag_set: HashSet<&str> = query_tags.iter().map(|s| s.as_str()).collect();
    let matching_tags = entry_tags
        .iter()
        .filter(|t| query_tag_set.contains(t.as_str()))
        .count();
    let tag_boost = (0.15 * matching_tags as f64).min(0.3);

    0.7 * lev + 0.3 * jac + tag_boost
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
    fn normalized_levenshtein_identical() {
        let sim = normalized_levenshtein_similarity("hello", "hello");
        assert!((sim - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn keyword_jaccard_partial_overlap() {
        let jac = keyword_jaccard(
            "Replace hooks with Rust",
            "Replace hooks with compiled Rust",
        );
        // intersection: {replace, hooks, with, rust} = 4
        // union: {replace, hooks, with, rust, compiled} = 5
        assert!((jac - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn keyword_jaccard_no_overlap() {
        let jac = keyword_jaccard("alpha beta", "gamma delta");
        assert!((jac - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn composite_score_tag_boost_capped() {
        let score = composite_score(
            "test",
            "test",
            &["a".into(), "b".into(), "c".into(), "d".into()],
            &["a".into(), "b".into(), "c".into(), "d".into()],
        );
        // lev = 1.0, jac = 1.0, tag_boost = min(0.3, 0.15*4=0.6) = 0.3
        // total = 0.7*1.0 + 0.3*1.0 + 0.3 = 1.3
        assert!((score - 1.3).abs() < 0.01);
    }

    #[test]
    fn composite_score_similar_titles() {
        let score = composite_score(
            "Replace hooks with compiled Rust",
            "Replace hooks with Rust binaries",
            &["rust".into(), "hooks".into()],
            &["rust".into(), "hooks".into()],
        );
        assert!(score >= 0.6, "expected >= 0.6, got {score}");
    }
}

//! Pure consolidation functions: similarity, recency, relevance scoring.

use std::collections::HashSet;

/// Generate all 3-character n-grams from a word.
fn char_3grams(word: &str) -> HashSet<String> {
    let chars: Vec<char> = word.chars().collect();
    let mut grams = HashSet::new();
    if chars.len() >= 3 {
        for i in 0..=(chars.len() - 3) {
            grams.insert(chars[i..i + 3].iter().collect());
        }
    }
    grams
}

/// Extract the set of word-level 3-grams from text.
///
/// Each word in the text produces its own 3-grams. For short words (<3 chars),
/// we use the word itself padded to make a single gram.
pub fn word_3grams(text: &str) -> HashSet<String> {
    let mut all = HashSet::new();
    for word in text.split_whitespace() {
        let word_lower = word.to_lowercase();
        if word_lower.len() >= 3 {
            for gram in char_3grams(&word_lower) {
                all.insert(format!("{word_lower}:{gram}"));
            }
        } else {
            // Short word: include as-is so it participates in similarity
            all.insert(word_lower.clone());
        }
    }
    all
}

/// Compute Jaccard similarity on word 3-gram sets.
///
/// Returns a value in [0.0, 1.0]. Identical strings → 1.0, disjoint → 0.0.
pub fn jaccard_3gram_similarity(a: &str, b: &str) -> f64 {
    let set_a = word_3grams(a);
    let set_b = word_3grams(b);

    if set_a.is_empty() && set_b.is_empty() {
        return 1.0;
    }

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        return 1.0;
    }

    intersection as f64 / union as f64
}

/// Linear decay over 1 year: max(0.0, 1.0 - age_days / 365.0)
pub fn recency_factor(age_days: u64) -> f64 {
    (1.0_f64 - age_days as f64 / 365.0_f64).max(0.0)
}

/// Compute a relevance score from recency and reference count.
///
/// Formula: `recency_factor * (1.0 + reference_count as f64 * 0.1)`
pub fn compute_relevance_score(recency: f64, reference_count: u32) -> f64 {
    recency * (1.0 + reference_count as f64 * 0.1)
}

/// Returns true when two entries should be merged (Jaccard similarity > 0.8).
pub fn should_merge(a: &str, b: &str) -> bool {
    jaccard_3gram_similarity(a, b) > 0.8
}

/// Returns true when an entry is too short for reliable dedup (< 10 words).
pub fn is_short_entry(content: &str) -> bool {
    content.split_whitespace().count() < 10
}

/// Returns true when an entry should be marked stale.
///
/// An entry is stale when it is older than 90 days AND has zero references.
pub fn should_mark_stale(age_days: u64, reference_count: u32) -> bool {
    age_days > 90 && reference_count == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-006: recency_factor boundary values
    #[test]
    fn test_recency_factor_zero_days() {
        let r = recency_factor(0);
        assert!((r - 1.0).abs() < f64::EPSILON, "expected 1.0, got {r}");
    }

    #[test]
    fn test_recency_factor_365_days() {
        let r = recency_factor(365);
        assert!(r.abs() < f64::EPSILON, "expected 0.0, got {r}");
    }

    #[test]
    fn test_recency_factor_180_days() {
        let r = recency_factor(180);
        // 1.0 - 180/365 ≈ 0.5068
        assert!((r - 0.5068).abs() < 0.001, "expected ~0.507, got {r}");
    }

    #[test]
    fn test_recency_factor_clamped_at_zero() {
        let r = recency_factor(400);
        assert!(r.abs() < f64::EPSILON, "expected 0.0, got {r}");
    }

    // PC-007: compute_relevance_score
    #[test]
    fn test_relevance_score_zero_age_zero_refs() {
        let score = compute_relevance_score(recency_factor(0), 0);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_relevance_score_zero_age_five_refs() {
        let score = compute_relevance_score(recency_factor(0), 5);
        // 1.0 * (1.0 + 5 * 0.1) = 1.5
        assert!((score - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_relevance_score_half_decay_no_refs() {
        let score = compute_relevance_score(0.5, 0);
        assert!((score - 0.5).abs() < f64::EPSILON);
    }

    // PC-008: jaccard_3gram_similarity
    #[test]
    fn test_jaccard_identical_strings() {
        let score = jaccard_3gram_similarity("hello world foo", "hello world foo");
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaccard_disjoint_strings() {
        let score = jaccard_3gram_similarity("aardvark elephant", "xyz qrs");
        assert!(
            score < 0.05,
            "expected near 0.0 for disjoint strings, got {score}"
        );
    }

    #[test]
    fn test_jaccard_empty_strings() {
        let score = jaccard_3gram_similarity("", "");
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaccard_partial_overlap() {
        let score = jaccard_3gram_similarity("hello world", "hello earth");
        assert!(
            score > 0.0 && score < 1.0,
            "expected partial overlap, got {score}"
        );
    }

    // PC-009: should_merge
    #[test]
    fn test_should_merge_identical_returns_true() {
        assert!(should_merge(
            "hello world foo bar baz",
            "hello world foo bar baz"
        ));
    }

    #[test]
    fn test_should_merge_disjoint_returns_false() {
        assert!(!should_merge(
            "aardvark elephant zebra",
            "quantum physics relativity"
        ));
    }

    #[test]
    fn test_should_merge_below_threshold_returns_false() {
        // Strings with low overlap should not merge
        assert!(!should_merge("cat", "dog"));
    }

    // PC-010: is_short_entry
    #[test]
    fn test_is_short_entry_nine_words() {
        let content = "one two three four five six seven eight nine";
        assert!(is_short_entry(content), "9 words should be short");
    }

    #[test]
    fn test_is_short_entry_ten_words() {
        let content = "one two three four five six seven eight nine ten";
        assert!(!is_short_entry(content), "10 words should not be short");
    }

    #[test]
    fn test_is_short_entry_empty() {
        assert!(is_short_entry(""), "empty should be short");
    }

    #[test]
    fn test_is_short_entry_single_word() {
        assert!(is_short_entry("hello"), "one word should be short");
    }

    // PC-011: should_mark_stale
    #[test]
    fn test_should_mark_stale_old_no_refs() {
        assert!(should_mark_stale(91, 0));
    }

    #[test]
    fn test_should_mark_stale_old_with_ref() {
        assert!(!should_mark_stale(91, 1));
    }

    #[test]
    fn test_should_mark_stale_young_no_refs() {
        assert!(!should_mark_stale(89, 0));
    }

    #[test]
    fn test_should_mark_stale_exactly_90_days() {
        // 90 days is NOT stale (strictly > 90)
        assert!(!should_mark_stale(90, 0));
    }
}

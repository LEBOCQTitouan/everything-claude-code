/// Line-by-line diff using the LCS (Longest Common Subsequence) algorithm.
/// Ported from TypeScript `smart-merge.ts::computeLineDiff`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffType {
    Same,
    Added,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffType,
    pub content: String,
}

/// Maximum cell count before falling back to a simpler line-based diff.
/// Prevents OOM on very large files (mirrors the 1M limit in TypeScript).
const MAX_MATRIX_CELLS: usize = 1_000_000;

/// Compute a line-level diff between `old` and `new` lines using LCS.
///
/// For inputs where `old.len() * new.len() > MAX_MATRIX_CELLS`, falls back
/// to a simple sequential comparison.
pub fn compute_line_diff(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    if old.len() * new.len() > MAX_MATRIX_CELLS {
        return simple_diff(old, new);
    }

    let m = old.len();
    let n = new.len();

    // Build LCS table
    let mut table = vec![vec![0u32; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                table[i][j] = table[i - 1][j - 1] + 1;
            } else {
                table[i][j] = table[i - 1][j].max(table[i][j - 1]);
            }
        }
    }

    // Backtrack to produce diff
    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
            result.push(DiffLine {
                kind: DiffType::Same,
                content: old[i - 1].to_string(),
            });
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || table[i][j - 1] >= table[i - 1][j]) {
            result.push(DiffLine {
                kind: DiffType::Added,
                content: new[j - 1].to_string(),
            });
            j -= 1;
        } else {
            result.push(DiffLine {
                kind: DiffType::Removed,
                content: old[i - 1].to_string(),
            });
            i -= 1;
        }
    }

    result.reverse();
    result
}

/// Simple fallback diff for very large inputs.
fn simple_diff(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    let mut result = Vec::new();
    let max_len = old.len().max(new.len());

    for i in 0..max_len {
        match (old.get(i), new.get(i)) {
            (Some(o), Some(n)) if *o == *n => {
                result.push(DiffLine {
                    kind: DiffType::Same,
                    content: o.to_string(),
                });
            }
            (Some(o), Some(n)) => {
                result.push(DiffLine {
                    kind: DiffType::Removed,
                    content: o.to_string(),
                });
                result.push(DiffLine {
                    kind: DiffType::Added,
                    content: n.to_string(),
                });
            }
            (Some(o), None) => {
                result.push(DiffLine {
                    kind: DiffType::Removed,
                    content: o.to_string(),
                });
            }
            (None, Some(n)) => {
                result.push(DiffLine {
                    kind: DiffType::Added,
                    content: n.to_string(),
                });
            }
            (None, None) => unreachable!(),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_input() {
        let lines = vec!["a", "b", "c"];
        let diff = compute_line_diff(&lines, &lines);
        assert!(diff.iter().all(|d| d.kind == DiffType::Same));
        assert_eq!(diff.len(), 3);
    }

    #[test]
    fn fully_added() {
        let diff = compute_line_diff(&[], &["a", "b"]);
        assert_eq!(diff.len(), 2);
        assert!(diff.iter().all(|d| d.kind == DiffType::Added));
    }

    #[test]
    fn fully_removed() {
        let diff = compute_line_diff(&["a", "b"], &[]);
        assert_eq!(diff.len(), 2);
        assert!(diff.iter().all(|d| d.kind == DiffType::Removed));
    }

    #[test]
    fn mixed_changes() {
        let old = vec!["a", "b", "c", "d"];
        let new = vec!["a", "x", "c", "d", "e"];
        let diff = compute_line_diff(&old, &new);

        let types: Vec<&DiffType> = diff.iter().map(|d| &d.kind).collect();
        // a=same, b=removed, x=added, c=same, d=same, e=added
        assert_eq!(
            types,
            vec![
                &DiffType::Same,
                &DiffType::Removed,
                &DiffType::Added,
                &DiffType::Same,
                &DiffType::Same,
                &DiffType::Added,
            ]
        );
    }

    #[test]
    fn single_line_change() {
        let old = vec!["hello"];
        let new = vec!["world"];
        let diff = compute_line_diff(&old, &new);
        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0].kind, DiffType::Removed);
        assert_eq!(diff[0].content, "hello");
        assert_eq!(diff[1].kind, DiffType::Added);
        assert_eq!(diff[1].content, "world");
    }

    #[test]
    fn empty_inputs() {
        let diff = compute_line_diff(&[], &[]);
        assert!(diff.is_empty());
    }
}

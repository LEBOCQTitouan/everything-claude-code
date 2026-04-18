//! Documentation coverage counting — detects `///` doc comments above `pub` items.

/// Per-module documentation coverage metrics.
#[derive(Debug, Clone, Default)]
pub struct ModuleCoverage {
    /// Module name.
    pub name: String,
    /// Total number of public items.
    pub total_pub_items: usize,
    /// Number of public items with doc comments.
    pub documented: usize,
    /// Coverage percentage (0.0 to 100.0).
    pub pct: f64,
}

/// Count documented vs total pub items in Rust source content.
pub fn count_doc_coverage(module_name: &str, content: &str) -> ModuleCoverage {
    let lines: Vec<&str> = content.lines().collect();
    let mut total = 0usize;
    let mut documented = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_pub_item(trimmed) {
            total += 1;
            // Check if immediately preceding lines are doc comments (contiguous)
            let has_doc = {
                let mut found = false;
                let mut j = i;
                while j > 0 {
                    j -= 1;
                    let prev = lines[j].trim();
                    if prev.starts_with("///") || prev.starts_with("#[doc") {
                        found = true;
                        break;
                    } else if prev.is_empty() || prev.starts_with("#[") {
                        // Skip blank lines and attributes between doc and item
                        continue;
                    } else {
                        break; // Non-doc, non-blank line = no doc for this item
                    }
                }
                found
            };
            if has_doc {
                documented += 1;
            }
        }
    }

    let pct = if total > 0 {
        (documented as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    ModuleCoverage {
        name: module_name.to_string(),
        total_pub_items: total,
        documented,
        pct,
    }
}

fn is_pub_item(line: &str) -> bool {
    (line.starts_with("pub fn ")
        || line.starts_with("pub struct ")
        || line.starts_with("pub enum ")
        || line.starts_with("pub trait ")
        || line.starts_with("pub type ")
        || line.starts_with("pub const ")
        || line.starts_with("pub static ")
        || line.starts_with("pub mod ")
        || line.starts_with("pub async fn "))
        && !line.starts_with("pub(crate)")
        && !line.starts_with("pub(super)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documented_pub_fn() {
        let src = "/// Does something.\npub fn foo() {}";
        let cov = count_doc_coverage("test", src);
        assert_eq!(cov.total_pub_items, 1);
        assert_eq!(cov.documented, 1);
        assert!((cov.pct - 100.0).abs() < 0.1);
    }

    #[test]
    fn undocumented_pub_fn() {
        let src = "pub fn foo() {}";
        let cov = count_doc_coverage("test", src);
        assert_eq!(cov.total_pub_items, 1);
        assert_eq!(cov.documented, 0);
        assert!((cov.pct - 0.0).abs() < 0.1);
    }

    #[test]
    fn mixed_coverage() {
        let src = "/// Documented.\npub fn a() {}\npub fn b() {}\n/// Also doc.\npub struct C;";
        let cov = count_doc_coverage("test", src);
        assert_eq!(cov.total_pub_items, 3);
        assert_eq!(cov.documented, 2);
    }

    #[test]
    fn pub_crate_excluded() {
        let src = "pub(crate) fn internal() {}";
        let cov = count_doc_coverage("test", src);
        assert_eq!(cov.total_pub_items, 0);
    }

    #[test]
    fn empty_file() {
        let cov = count_doc_coverage("test", "");
        assert_eq!(cov.total_pub_items, 0);
        assert!((cov.pct - 0.0).abs() < 0.1);
    }
}

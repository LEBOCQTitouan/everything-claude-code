use ecc_domain::config::validate::extract_frontmatter;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use rayon::prelude::*;
use std::path::Path;

use super::code_block_scanning::scan_unsafe_code;
use super::cross_ref_validation::validate_cross_refs;
use super::frontmatter_validation::{
    validate_difficulty, validate_frontmatter_fields, validate_languages,
};
use super::section_validation::validate_sections;

/// Context passed to per-file validation helpers to reduce parameter count.
struct ValidationCtx<'a> {
    label: &'a str,
    stem: &'a str,
    expected_category: &'a str,
    content: &'a str,
    all_stems: &'a [String],
}

pub(super) fn validate_patterns(
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
) -> bool {
    let patterns_dir = root.join("patterns");
    if !fs.exists(&patterns_dir) {
        terminal.stdout_write("No patterns directory found, skipping validation\n");
        return true;
    }

    let root_entries = match fs.read_dir(&patterns_dir) {
        Ok(e) => e,
        Err(err) => {
            terminal.stderr_write(&format!("ERROR: Cannot read patterns directory: {err}\n"));
            return false;
        }
    };

    warn_root_level_files(&root_entries, fs, terminal);

    let categories: Vec<_> = root_entries.into_iter().filter(|p| fs.is_dir(p)).collect();
    let index_content = read_index_content(&patterns_dir, fs);
    let all_stems = collect_pattern_stems(&categories, fs);

    // Collect per-file work items for parallel validation
    let work_items = collect_work_items(&categories, fs);

    // Parallel pattern file validation via rayon
    let results: Vec<(String, String, String, bool)> = work_items
        .par_iter()
        .map(|(category_name, file_path)| {
            let stem = file_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let file_label = format!(
                "{}/{}",
                category_name,
                file_path.file_name().unwrap_or_default().to_string_lossy()
            );
            let content = match fs.read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => {
                    return (file_label, stem, format!("ERROR: read failed - {e}"), false);
                }
            };

            let mut errors = String::new();
            let mut has_errors = false;

            if !index_content.is_empty() && !stem_in_index(&index_content, &stem) {
                errors.push_str(&format!(
                    "ERROR: {file_label} - pattern '{stem}' is not listed in patterns/index.md\n"
                ));
                has_errors = true;
            }

            let ctx = ValidationCtx {
                label: &file_label,
                stem: &stem,
                expected_category: category_name,
                content: &content,
                all_stems: &all_stems,
            };
            let (file_errors, file_ok) = validate_pattern_file(&ctx);
            errors.push_str(&file_errors);
            if !file_ok {
                has_errors = true;
            }

            (file_label, stem, errors, !has_errors)
        })
        .collect();

    // Aggregate results sequentially (terminal output must be serial)
    let mut has_errors = false;
    let mut file_count: usize = 0;
    for (_label, _stem, errors, ok) in &results {
        file_count += 1;
        if !errors.is_empty() {
            terminal.stderr_write(errors);
        }
        if !ok {
            has_errors = true;
        }
    }

    if has_errors {
        return false;
    }

    let category_count = categories
        .iter()
        .filter(|cat| {
            fs.read_dir(cat)
                .map(|entries| {
                    entries.iter().any(|p| {
                        p.extension()
                            .map(|ext| ext.eq_ignore_ascii_case("md"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        })
        .count();

    terminal.stdout_write(&format!(
        "Validated {} pattern files across {} categories\n",
        file_count, category_count
    ));
    true
}

/// Warn about root-level .md files (not in a category subdir), skip them.
fn warn_root_level_files(
    entries: &[std::path::PathBuf],
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
) {
    for entry in entries {
        if !fs.is_dir(entry)
            && entry
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
        {
            let fname = entry
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if fname != "index.md" {
                terminal.stderr_write(&format!(
                    "WARN: {fname} - root-level .md file in patterns/ is not in a category subdirectory, skipping\n"
                ));
            }
        }
    }
}

/// Read the index.md content for coverage checking.
fn read_index_content(patterns_dir: &Path, fs: &dyn FileSystem) -> String {
    let index_path = patterns_dir.join("index.md");
    if fs.exists(&index_path) {
        fs.read_to_string(&index_path).unwrap_or_default()
    } else {
        String::new()
    }
}

/// Collect all pattern stems across all categories for cross-ref resolution.
///
/// For the `idioms` category, recurses one level into language subdirectories
/// (e.g., `idioms/rust/`, `idioms/go/`) and collects stems from those subdirs.
/// Stems include both bare names (`newtype`) and category-prefixed variants
/// (`idioms/newtype`) for disambiguation.
fn collect_pattern_stems(categories: &[std::path::PathBuf], fs: &dyn FileSystem) -> Vec<String> {
    let mut all_stems: Vec<String> = Vec::new();
    for category_path in categories {
        let category_name = category_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dirs_to_scan = if category_name == "idioms" {
            collect_idiom_subdirs(category_path, fs)
        } else {
            vec![category_path.clone()]
        };
        for dir in &dirs_to_scan {
            if let Ok(files) = fs.read_dir(dir) {
                for file_path in files {
                    if file_path
                        .extension()
                        .map(|ext| ext.eq_ignore_ascii_case("md"))
                        .unwrap_or(false)
                        && let Some(stem) = file_path.file_stem()
                    {
                        let stem_str = stem.to_string_lossy().to_string();
                        // Add category-prefixed variant for disambiguation
                        all_stems.push(format!("{category_name}/{stem_str}"));
                        all_stems.push(stem_str);
                    }
                }
            }
        }
    }
    all_stems
}

/// Collect language subdirectories within the idioms category.
fn collect_idiom_subdirs(
    idioms_path: &std::path::Path,
    fs: &dyn FileSystem,
) -> Vec<std::path::PathBuf> {
    let mut subdirs = Vec::new();
    if let Ok(entries) = fs.read_dir(idioms_path) {
        for entry in entries {
            if fs.is_dir(&entry) {
                subdirs.push(entry);
            }
        }
    }
    subdirs
}

/// Collect (category_name, file_path) pairs for all .md files in category dirs.
///
/// For the `idioms` category, recurses into language subdirectories and uses
/// `idioms` as the category_name for all nested files (matching frontmatter convention).
fn collect_work_items(
    categories: &[std::path::PathBuf],
    fs: &dyn FileSystem,
) -> Vec<(String, std::path::PathBuf)> {
    let mut items = Vec::new();
    for category_path in categories {
        let category_name = category_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dirs_to_scan = if category_name == "idioms" {
            collect_idiom_subdirs(category_path, fs)
        } else {
            vec![category_path.clone()]
        };
        for dir in &dirs_to_scan {
            if let Ok(entries) = fs.read_dir(dir) {
                for file_path in entries {
                    if file_path
                        .extension()
                        .map(|ext| ext.eq_ignore_ascii_case("md"))
                        .unwrap_or(false)
                    {
                        items.push((category_name.clone(), file_path));
                    }
                }
            }
        }
    }
    items
}

/// Check whether stem appears in the index as a path component (word-boundary-aware).
///
/// Matches patterns like `/factory-method.md`, `(factory-method)`, or the stem
/// preceded by a word boundary character and followed by `.md`, `)`, or whitespace.
pub(crate) fn stem_in_index(index_content: &str, stem: &str) -> bool {
    let boundary_before =
        |c: char| -> bool { matches!(c, '/' | '(' | '[' | ' ' | '\t' | '\n' | '-') || c == '`' };
    let boundary_after = |s: &str| -> bool {
        s.is_empty()
            || s.starts_with(".md")
            || s.starts_with(')')
            || s.starts_with(']')
            || s.starts_with(char::is_whitespace)
            || s.starts_with('`')
    };

    let mut search_from = 0;
    while let Some(pos) = index_content[search_from..].find(stem) {
        let abs_pos = search_from + pos;
        let before_ok =
            abs_pos == 0 || boundary_before(index_content.as_bytes()[abs_pos - 1] as char);
        let after_start = abs_pos + stem.len();
        let after_ok = boundary_after(&index_content[after_start..]);
        if before_ok && after_ok {
            return true;
        }
        search_from = abs_pos + 1;
    }
    false
}

/// Validate a single pattern file. Returns (error_messages, is_valid).
fn validate_pattern_file(ctx: &ValidationCtx<'_>) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    let fm = match extract_frontmatter(ctx.content) {
        Some(map) => map,
        None => {
            return (
                format!("ERROR: {} - No frontmatter found\n", ctx.label),
                false,
            );
        }
    };

    let (fm_errs, fm_ok) = validate_frontmatter_fields(&fm, ctx.label, ctx.expected_category);
    errors.push_str(&fm_errs);
    if !fm_ok {
        has_errors = true;
    }

    let (lang_errs, lang_ok) = validate_languages(&fm, ctx.content, ctx.label);
    errors.push_str(&lang_errs);
    if !lang_ok {
        has_errors = true;
    }

    let (diff_errs, diff_ok) = validate_difficulty(&fm, ctx.label);
    errors.push_str(&diff_errs);
    if !diff_ok {
        has_errors = true;
    }

    let (ref_errs, ref_ok) = validate_cross_refs(&fm, ctx.stem, ctx.all_stems, ctx.label);
    errors.push_str(&ref_errs);
    if !ref_ok {
        has_errors = true;
    }

    errors.push_str(&scan_unsafe_code(&fm, ctx.content, ctx.label));

    // Warn (but don't error) if file exceeds recommended size
    let line_count = ctx.content.lines().count();
    if line_count > ecc_domain::config::validate::PATTERN_SIZE_WARNING_LINES {
        errors.push_str(&format!(
            "WARN: {} - File has {} lines (exceeds {} recommended max)\n",
            ctx.label,
            line_count,
            ecc_domain::config::validate::PATTERN_SIZE_WARNING_LINES,
        ));
    }

    let (sec_errs, sec_ok) = validate_sections(ctx.content, ctx.label);
    errors.push_str(&sec_errs);
    if !sec_ok {
        has_errors = true;
    }

    (errors, !has_errors)
}

/// Generate patterns/index.md from pattern file frontmatter.
///
/// Called when `--fix` is passed to `ecc validate patterns`.
/// Scans all pattern files, extracts frontmatter, and writes a structured index
/// with category headers, pattern links, language coverage table,
/// alphabetized tag list with occurrence counts, and total pattern count.
pub(super) fn generate_index(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) {
    use ecc_domain::config::validate::extract_frontmatter;
    use std::collections::BTreeMap;

    let patterns_dir = root.join("patterns");
    if !fs.exists(&patterns_dir) {
        return;
    }

    let root_entries = match fs.read_dir(&patterns_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let categories: Vec<_> = root_entries.into_iter().filter(|p| fs.is_dir(p)).collect();

    // Collect all pattern info grouped by category
    let mut category_patterns: BTreeMap<String, Vec<(String, String, Vec<String>)>> =
        BTreeMap::new();
    let mut language_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_count: usize = 0;

    let work_items = collect_work_items(&categories, fs);

    for (category_name, file_path) in &work_items {
        let content = match fs.read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let fm = match extract_frontmatter(&content) {
            Some(m) => m,
            None => continue,
        };

        let name = fm.get("name").cloned().unwrap_or_default();
        let languages: Vec<String> = fm
            .get("languages")
            .map(|l| {
                ecc_domain::config::validate::parse_tool_list(l.trim())
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        let tags: Vec<String> = fm
            .get("tags")
            .map(|t| {
                ecc_domain::config::validate::parse_tool_list(t.trim())
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        // Track language counts
        for lang in &languages {
            *language_counts.entry(lang.clone()).or_insert(0) += 1;
        }

        // Track tag counts
        for tag in &tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }

        // Compute relative path from patterns/ to this file
        let rel_path = file_path
            .strip_prefix(&patterns_dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        category_patterns
            .entry(category_name.clone())
            .or_default()
            .push((name, rel_path, languages));

        total_count += 1;
    }

    // Generate index content
    let mut index = String::new();
    index.push_str("# Pattern Library Index\n\n");

    // Categories with pattern links
    index.push_str("## Categories\n\n");
    for (category, patterns) in &category_patterns {
        let title = category
            .split('-')
            .map(|w| {
                let mut chars = w.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        index.push_str(&format!("### {} ({} patterns)\n\n", title, patterns.len()));
        for (name, rel_path, _) in patterns {
            let display_name = name
                .split('-')
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            index.push_str(&format!("- [{display_name}]({rel_path})\n"));
        }
        index.push('\n');
    }

    // Language coverage table
    index.push_str("## Language Coverage\n\n");
    index.push_str("| Language | Pattern Count |\n");
    index.push_str("|----------|---------------|\n");
    for (lang, count) in &language_counts {
        index.push_str(&format!("| {lang} | {count} |\n"));
    }
    index.push('\n');

    // Tag list with counts
    index.push_str("## Tags\n\n");
    for (tag, count) in &tag_counts {
        index.push_str(&format!("- `{tag}` ({count})\n"));
    }
    index.push('\n');

    // Total count
    index.push_str(&format!("**Total patterns: {total_count}**\n"));

    // Write index
    let index_path = patterns_dir.join("index.md");
    match fs.write(&index_path, &index) {
        Ok(()) => {
            terminal.stdout_write(&format!(
                "Generated patterns/index.md ({total_count} patterns across {} categories)\n",
                category_patterns.len()
            ));
        }
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Failed to write patterns/index.md: {e}\n"));
        }
    }
}

#[cfg(test)]
#[path = "patterns_tests.rs"]
mod tests_module;

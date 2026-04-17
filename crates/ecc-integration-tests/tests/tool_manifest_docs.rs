mod common;

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

// ── PC-058: competitor_research_claim_updated ─────────────────────────────────

/// The competitor research file must contain the updated claim
/// "ECC: declarative tool manifest" (not the old "ECC: hardcoded allowedTools").
#[test]
fn competitor_research_claim_updated() {
    let root = workspace_root();
    let path = root.join("docs/research/competitor-claw-goose.md");

    assert!(
        path.exists(),
        "docs/research/competitor-claw-goose.md must exist"
    );

    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read competitor-claw-goose.md: {e}"));

    assert!(
        content.contains("ECC: declarative tool manifest"),
        "competitor-claw-goose.md must contain 'ECC: declarative tool manifest' but got:\n{content}"
    );

    assert!(
        !content.contains("ECC: hardcoded allowedTools"),
        "competitor-claw-goose.md must NOT contain old claim 'ECC: hardcoded allowedTools'"
    );
}

// ── PC-059: adr_exists_with_required_cites ────────────────────────────────────

/// ADR 0060 must exist and contain references to BL-146, BL-140, and
/// competitor-claw-goose.
#[test]
fn adr_exists_with_required_cites() {
    let root = workspace_root();
    let path = root.join("docs/adr/0060-declarative-tool-manifest.md");

    assert!(
        path.exists(),
        "docs/adr/0060-declarative-tool-manifest.md must exist"
    );

    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read ADR 0060: {e}"));

    assert!(
        content.contains("BL-146"),
        "ADR 0060 must cite BL-146 but content does not contain it"
    );

    assert!(
        content.contains("BL-140"),
        "ADR 0060 must cite BL-140 but content does not contain it"
    );

    assert!(
        content.contains("competitor-claw-goose"),
        "ADR 0060 must cite competitor-claw-goose but content does not contain it"
    );
}

// ── PC-060: authoring_guide_exists_with_sections ─────────────────────────────

/// docs/tool-manifest-authoring.md must exist with the required sections.
#[test]
fn authoring_guide_exists_with_sections() {
    let root = workspace_root();
    let path = root.join("docs/tool-manifest-authoring.md");

    assert!(
        path.exists(),
        "docs/tool-manifest-authoring.md must exist"
    );

    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read tool-manifest-authoring.md: {e}"));

    assert!(
        content.contains("# Adding a Tool"),
        "tool-manifest-authoring.md must contain '# Adding a Tool'"
    );

    assert!(
        content.contains("# Adding a Preset"),
        "tool-manifest-authoring.md must contain '# Adding a Preset'"
    );
}

// ── PC-061: claude_md_gotcha_and_glossary ────────────────────────────────────

/// CLAUDE.md must have exactly one new gotcha line for tool-set and the
/// glossary must contain the 3-term extension (tool-set, install-time expansion,
/// manifest).
#[test]
fn claude_md_gotcha_and_glossary() {
    let root = workspace_root();
    let path = root.join("CLAUDE.md");

    assert!(path.exists(), "CLAUDE.md must exist");

    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read CLAUDE.md: {e}"));

    // The glossary line must mention tool-set
    assert!(
        content.contains("**tool-set**"),
        "CLAUDE.md glossary must contain '**tool-set**'"
    );

    // Must mention manifest/tool-manifest.yaml
    assert!(
        content.contains("manifest/tool-manifest.yaml"),
        "CLAUDE.md must reference 'manifest/tool-manifest.yaml' for tool-set definition"
    );

    // Must mention install time expansion
    assert!(
        content.contains("install time") || content.contains("install-time") || content.contains("at install"),
        "CLAUDE.md must mention install-time expansion for tool-set"
    );
}

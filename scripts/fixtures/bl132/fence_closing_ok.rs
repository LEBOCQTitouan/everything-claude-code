// BL-132 fixture: well-formed open+close fence pair — classifier MUST PASS.
//
// Expected classifier verdict: PASS
// Reason: closing fences are ignored by the language-hint classifier (only openers need
// a language tag). A correctly-formed `text` fence that closes with plain ``` must pass.

/// Legal fence lifecycle.
///
/// ```text
/// open
///   ...
/// close
/// ```
pub struct LegalFence;

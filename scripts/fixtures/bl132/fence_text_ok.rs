// BL-132 fixture: fence opener tagged `text` — classifier MUST PASS.
//
// Expected classifier verdict: PASS
// Reason: every opening fence declares one of the allowed languages (text/rust/ignore/no_run/compile_fail).

/// State transition diagram.
///
/// ```text
/// [Idle] --> [Running] --> [Done]
/// ```
pub enum State {
    Idle,
    Running,
    Done,
}

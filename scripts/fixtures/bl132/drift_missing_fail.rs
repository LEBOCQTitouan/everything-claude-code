// BL-132 fixture: flow/decision diagram WITHOUT drift anchor — classifier MUST FAIL.
//
// Expected classifier verdict: FAIL
// Reason: the diagram contains `--Y-->` and `--N-->` tokens (flow/decision) but has no
// `<!-- keep in sync with: <test_fn_name> -->` comment in the 3 `///` lines preceding
// the fence opener. R-3 AC-R3.1 requires the anchor on every flow diagram.

/// Evaluate input.
///
/// ```text
/// [input?] --Y--> [ok]
///          --N--> [err]
/// ```
pub fn evaluate(x: bool) -> bool {
    x
}

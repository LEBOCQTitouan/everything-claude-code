// BL-132 fixture: flow/decision diagram WITH drift anchor — classifier MUST PASS.
//
// Expected classifier verdict: PASS
// Reason: the diagram uses `--Y-->` tokens, but the `///` line 2 positions above the
// fence opener contains `<!-- keep in sync with: evaluate_flow -->`, satisfying R-3.

/// Evaluate input.
///
/// <!-- keep in sync with: evaluate_flow -->
/// ```text
/// [input?] --Y--> [ok]
///          --N--> [err]
/// ```
pub fn evaluate(x: bool) -> bool {
    x
}

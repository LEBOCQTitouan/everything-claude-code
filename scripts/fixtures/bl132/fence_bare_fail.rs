// BL-132 fixture: fence opener WITHOUT language hint — classifier MUST FAIL.
//
// Expected classifier verdict: FAIL
// Reason: a bare ``` opener defaults rustdoc to `rust` and triggers doctest compilation
// of the ASCII content, which fails the doc build. R-1 AC-R1.2 bans this.

/// Accidental bare fence below.
///
/// ```
/// [a] --> [b]
/// ```
pub struct Placeholder;

//! Metric event types.

use std::fmt;

use super::error::MetricsError;

/// The type of harness operation that was measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricEventType {
    /// A hook script execution.
    HookExecution,
    /// A workflow phase transition.
    PhaseTransition,
    /// An agent subagent spawn.
    AgentSpawn,
    /// A commit gate check (build/test/lint).
    CommitGate,
}

impl fmt::Display for MetricEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HookExecution => write!(f, "hook_execution"),
            Self::PhaseTransition => write!(f, "phase_transition"),
            Self::AgentSpawn => write!(f, "agent_spawn"),
            Self::CommitGate => write!(f, "commit_gate"),
        }
    }
}

impl MetricEventType {
    /// Parse from a string representation.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "hook_execution" => Some(Self::HookExecution),
            "phase_transition" => Some(Self::PhaseTransition),
            "agent_spawn" => Some(Self::AgentSpawn),
            "commit_gate" => Some(Self::CommitGate),
            _ => None,
        }
    }
}

/// The outcome of a harness operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricOutcome {
    /// Operation completed successfully.
    Success,
    /// Operation failed.
    Failure,
    /// State transition was rejected by validation.
    Rejected,
    /// Commit gate check passed.
    Passed,
}

impl fmt::Display for MetricOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Rejected => write!(f, "rejected"),
            Self::Passed => write!(f, "passed"),
        }
    }
}

impl MetricOutcome {
    /// Parse from a string representation.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "success" => Some(Self::Success),
            "failure" => Some(Self::Failure),
            "rejected" => Some(Self::Rejected),
            "passed" => Some(Self::Passed),
            _ => None,
        }
    }
}

/// Which commit gate failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitGateKind {
    /// Build compilation check.
    Build,
    /// Test suite execution.
    Test,
    /// Lint/clippy check.
    Lint,
}

impl fmt::Display for CommitGateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build => write!(f, "build"),
            Self::Test => write!(f, "test"),
            Self::Lint => write!(f, "lint"),
        }
    }
}

impl CommitGateKind {
    /// Parse from a string representation.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "build" => Some(Self::Build),
            "test" => Some(Self::Test),
            "lint" => Some(Self::Lint),
            _ => None,
        }
    }
}

/// A single recorded harness operation outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricEvent {
    /// Optional database row id (None before persistence).
    pub id: Option<i64>,
    /// The type of operation.
    pub event_type: MetricEventType,
    /// Session identifier.
    pub session_id: String,
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// The outcome of the operation.
    pub outcome: MetricOutcome,
    /// Hook ID (only for HookExecution).
    pub hook_id: Option<String>,
    /// Duration in milliseconds (only for HookExecution).
    pub duration_ms: Option<u64>,
    /// Error message (for failures).
    pub error_message: Option<String>,
    /// Source phase (only for PhaseTransition).
    pub from_phase: Option<String>,
    /// Target phase (only for PhaseTransition).
    pub to_phase: Option<String>,
    /// Rejection reason (only for PhaseTransition with Rejected outcome).
    pub rejection_reason: Option<String>,
    /// Agent type string (only for AgentSpawn).
    pub agent_type: Option<String>,
    /// Number of retries (only for AgentSpawn).
    pub retry_count: Option<u32>,
    /// Which gates failed (only for CommitGate).
    pub gates_failed: Vec<CommitGateKind>,
}

/// Validate that the outcome is valid for the given event type.
fn validate_outcome(
    event_type: MetricEventType,
    outcome: MetricOutcome,
) -> Result<(), MetricsError> {
    let valid = match event_type {
        MetricEventType::HookExecution => {
            matches!(outcome, MetricOutcome::Success | MetricOutcome::Failure)
        }
        MetricEventType::PhaseTransition => {
            matches!(outcome, MetricOutcome::Success | MetricOutcome::Rejected)
        }
        MetricEventType::AgentSpawn => {
            matches!(outcome, MetricOutcome::Success | MetricOutcome::Failure)
        }
        MetricEventType::CommitGate => {
            matches!(outcome, MetricOutcome::Passed | MetricOutcome::Failure)
        }
    };
    if valid {
        Ok(())
    } else {
        Err(MetricsError::InvalidOutcome {
            event_type: event_type.to_string(),
            outcome: outcome.to_string(),
        })
    }
}

impl MetricEvent {
    /// Create a hook execution metric event.
    pub fn hook_execution(
        session_id: String,
        timestamp: String,
        hook_id: String,
        duration_ms: u64,
        outcome: MetricOutcome,
        error_message: Option<String>,
    ) -> Result<Self, MetricsError> {
        validate_outcome(MetricEventType::HookExecution, outcome)?;
        Ok(Self {
            id: None,
            event_type: MetricEventType::HookExecution,
            session_id,
            timestamp,
            outcome,
            hook_id: Some(hook_id),
            duration_ms: Some(duration_ms),
            error_message,
            from_phase: None,
            to_phase: None,
            rejection_reason: None,
            agent_type: None,
            retry_count: None,
            gates_failed: Vec::new(),
        })
    }

    /// Create a phase transition metric event.
    pub fn phase_transition(
        session_id: String,
        timestamp: String,
        from_phase: String,
        to_phase: String,
        outcome: MetricOutcome,
        rejection_reason: Option<String>,
    ) -> Result<Self, MetricsError> {
        validate_outcome(MetricEventType::PhaseTransition, outcome)?;
        Ok(Self {
            id: None,
            event_type: MetricEventType::PhaseTransition,
            session_id,
            timestamp,
            outcome,
            hook_id: None,
            duration_ms: None,
            error_message: None,
            from_phase: Some(from_phase),
            to_phase: Some(to_phase),
            rejection_reason,
            agent_type: None,
            retry_count: None,
            gates_failed: Vec::new(),
        })
    }

    /// Create an agent spawn metric event.
    pub fn agent_spawn(
        session_id: String,
        timestamp: String,
        agent_type: String,
        outcome: MetricOutcome,
        retry_count: Option<u32>,
    ) -> Result<Self, MetricsError> {
        validate_outcome(MetricEventType::AgentSpawn, outcome)?;
        Ok(Self {
            id: None,
            event_type: MetricEventType::AgentSpawn,
            session_id,
            timestamp,
            outcome,
            hook_id: None,
            duration_ms: None,
            error_message: None,
            from_phase: None,
            to_phase: None,
            rejection_reason: None,
            agent_type: Some(agent_type),
            retry_count,
            gates_failed: Vec::new(),
        })
    }

    /// Create a commit gate metric event.
    pub fn commit_gate(
        session_id: String,
        timestamp: String,
        outcome: MetricOutcome,
        gates_failed: Vec<CommitGateKind>,
    ) -> Result<Self, MetricsError> {
        validate_outcome(MetricEventType::CommitGate, outcome)?;
        Ok(Self {
            id: None,
            event_type: MetricEventType::CommitGate,
            session_id,
            timestamp,
            outcome,
            hook_id: None,
            duration_ms: None,
            error_message: None,
            from_phase: None,
            to_phase: None,
            rejection_reason: None,
            agent_type: None,
            retry_count: None,
            gates_failed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-001: MetricEventType display
    #[test]
    fn metric_event_type_display() {
        assert_eq!(MetricEventType::HookExecution.to_string(), "hook_execution");
        assert_eq!(
            MetricEventType::PhaseTransition.to_string(),
            "phase_transition"
        );
        assert_eq!(MetricEventType::AgentSpawn.to_string(), "agent_spawn");
        assert_eq!(MetricEventType::CommitGate.to_string(), "commit_gate");

        assert_eq!(
            MetricEventType::from_str_opt("hook_execution"),
            Some(MetricEventType::HookExecution)
        );
        assert_eq!(
            MetricEventType::from_str_opt("phase_transition"),
            Some(MetricEventType::PhaseTransition)
        );
        assert_eq!(
            MetricEventType::from_str_opt("agent_spawn"),
            Some(MetricEventType::AgentSpawn)
        );
        assert_eq!(
            MetricEventType::from_str_opt("commit_gate"),
            Some(MetricEventType::CommitGate)
        );
        assert_eq!(MetricEventType::from_str_opt("invalid"), None);
    }

    // PC-002: MetricOutcome variants
    #[test]
    fn metric_outcome_variants() {
        assert_eq!(MetricOutcome::Success.to_string(), "success");
        assert_eq!(MetricOutcome::Failure.to_string(), "failure");
        assert_eq!(MetricOutcome::Rejected.to_string(), "rejected");
        assert_eq!(MetricOutcome::Passed.to_string(), "passed");

        assert_eq!(
            MetricOutcome::from_str_opt("success"),
            Some(MetricOutcome::Success)
        );
        assert_eq!(
            MetricOutcome::from_str_opt("failure"),
            Some(MetricOutcome::Failure)
        );
        assert_eq!(
            MetricOutcome::from_str_opt("rejected"),
            Some(MetricOutcome::Rejected)
        );
        assert_eq!(
            MetricOutcome::from_str_opt("passed"),
            Some(MetricOutcome::Passed)
        );
        assert_eq!(MetricOutcome::from_str_opt("bogus"), None);
    }

    // PC-003: CommitGateKind variants
    #[test]
    fn commit_gate_kind_variants() {
        assert_eq!(CommitGateKind::Build.to_string(), "build");
        assert_eq!(CommitGateKind::Test.to_string(), "test");
        assert_eq!(CommitGateKind::Lint.to_string(), "lint");

        assert_eq!(
            CommitGateKind::from_str_opt("build"),
            Some(CommitGateKind::Build)
        );
        assert_eq!(
            CommitGateKind::from_str_opt("test"),
            Some(CommitGateKind::Test)
        );
        assert_eq!(
            CommitGateKind::from_str_opt("lint"),
            Some(CommitGateKind::Lint)
        );
        assert_eq!(CommitGateKind::from_str_opt("nope"), None);
    }

    // PC-004: hook_execution event
    #[test]
    fn hook_execution_event() {
        let event = MetricEvent::hook_execution(
            "sess-1".into(),
            "2026-04-06T10:00:00Z".into(),
            "pre:edit:fmt".into(),
            42,
            MetricOutcome::Success,
            None,
        )
        .unwrap();

        assert_eq!(event.event_type, MetricEventType::HookExecution);
        assert_eq!(event.session_id, "sess-1");
        assert_eq!(event.hook_id.as_deref(), Some("pre:edit:fmt"));
        assert_eq!(event.duration_ms, Some(42));
        assert_eq!(event.outcome, MetricOutcome::Success);
        assert!(event.error_message.is_none());
    }

    // PC-005: phase_transition event
    #[test]
    fn phase_transition_event() {
        let event = MetricEvent::phase_transition(
            "sess-1".into(),
            "2026-04-06T10:00:00Z".into(),
            "plan".into(),
            "solution".into(),
            MetricOutcome::Success,
            None,
        )
        .unwrap();

        assert_eq!(event.event_type, MetricEventType::PhaseTransition);
        assert_eq!(event.from_phase.as_deref(), Some("plan"));
        assert_eq!(event.to_phase.as_deref(), Some("solution"));
    }

    // PC-006: agent_spawn event
    #[test]
    fn agent_spawn_event() {
        let event = MetricEvent::agent_spawn(
            "sess-1".into(),
            "2026-04-06T10:00:00Z".into(),
            "code-reviewer".into(),
            MetricOutcome::Failure,
            Some(2),
        )
        .unwrap();

        assert_eq!(event.event_type, MetricEventType::AgentSpawn);
        assert_eq!(event.agent_type.as_deref(), Some("code-reviewer"));
        assert_eq!(event.retry_count, Some(2));
        assert_eq!(event.outcome, MetricOutcome::Failure);
    }

    // PC-007: commit_gate event
    #[test]
    fn commit_gate_event() {
        let event = MetricEvent::commit_gate(
            "sess-1".into(),
            "2026-04-06T10:00:00Z".into(),
            MetricOutcome::Failure,
            vec![CommitGateKind::Build, CommitGateKind::Lint],
        )
        .unwrap();

        assert_eq!(event.event_type, MetricEventType::CommitGate);
        assert_eq!(event.outcome, MetricOutcome::Failure);
        assert_eq!(event.gates_failed.len(), 2);
        assert_eq!(event.gates_failed[0], CommitGateKind::Build);
        assert_eq!(event.gates_failed[1], CommitGateKind::Lint);
    }

    // PC-008: invalid outcome for event type
    #[test]
    fn invalid_outcome_for_event_type() {
        assert!(MetricEvent::hook_execution(
            "s".into(), "t".into(), "h".into(), 0,
            MetricOutcome::Rejected, None,
        ).is_err());

        assert!(MetricEvent::phase_transition(
            "s".into(), "t".into(), "a".into(), "b".into(),
            MetricOutcome::Passed, None,
        ).is_err());

        assert!(MetricEvent::agent_spawn(
            "s".into(), "t".into(), "a".into(),
            MetricOutcome::Rejected, None,
        ).is_err());

        assert!(MetricEvent::commit_gate(
            "s".into(), "t".into(),
            MetricOutcome::Success, vec![],
        ).is_err());

        // CommitGate CAN have Passed outcome
        assert!(MetricEvent::commit_gate(
            "s".into(), "t".into(),
            MetricOutcome::Passed, vec![],
        ).is_ok());
    }
}

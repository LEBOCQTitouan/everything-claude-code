//! Install helpers — artifact collection, settings management, and summary display.

mod artifacts;
pub(super) mod rule_filter;
mod settings;
pub(super) mod stack_detect;
mod summary;

pub(super) use artifacts::{collect_installed_artifacts, collect_rule_groups};
pub(super) use settings::{ensure_deny_rules_in_settings, ensure_statusline_in_settings};
pub(super) use summary::print_summary;

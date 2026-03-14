/// Available Claude models for Claw sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClawModel {
    #[default]
    Sonnet,
    Opus,
    Haiku,
}

impl ClawModel {
    /// Parse a model name (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "sonnet" => Some(Self::Sonnet),
            "opus" => Some(Self::Opus),
            "haiku" => Some(Self::Haiku),
            _ => None,
        }
    }

    /// The model flag value for `claude -p --model`.
    pub fn to_flag(&self) -> &'static str {
        match self {
            Self::Sonnet => "sonnet",
            Self::Opus => "opus",
            Self::Haiku => "haiku",
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sonnet => "Sonnet",
            Self::Opus => "Opus",
            Self::Haiku => "Haiku",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sonnet() {
        assert_eq!(ClawModel::parse("sonnet"), Some(ClawModel::Sonnet));
    }

    #[test]
    fn parse_opus() {
        assert_eq!(ClawModel::parse("opus"), Some(ClawModel::Opus));
    }

    #[test]
    fn parse_haiku() {
        assert_eq!(ClawModel::parse("haiku"), Some(ClawModel::Haiku));
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(ClawModel::parse("SONNET"), Some(ClawModel::Sonnet));
        assert_eq!(ClawModel::parse("Opus"), Some(ClawModel::Opus));
        assert_eq!(ClawModel::parse("HAIKU"), Some(ClawModel::Haiku));
    }

    #[test]
    fn parse_with_whitespace() {
        assert_eq!(ClawModel::parse("  sonnet  "), Some(ClawModel::Sonnet));
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(ClawModel::parse("gpt4"), None);
        assert_eq!(ClawModel::parse(""), None);
    }

    #[test]
    fn to_flag_values() {
        assert_eq!(ClawModel::Sonnet.to_flag(), "sonnet");
        assert_eq!(ClawModel::Opus.to_flag(), "opus");
        assert_eq!(ClawModel::Haiku.to_flag(), "haiku");
    }

    #[test]
    fn display_names() {
        assert_eq!(ClawModel::Sonnet.display_name(), "Sonnet");
        assert_eq!(ClawModel::Opus.display_name(), "Opus");
        assert_eq!(ClawModel::Haiku.display_name(), "Haiku");
    }

    #[test]
    fn default_is_sonnet() {
        assert_eq!(ClawModel::default(), ClawModel::Sonnet);
    }
}

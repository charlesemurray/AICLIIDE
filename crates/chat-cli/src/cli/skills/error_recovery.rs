/// Error recovery guidance for skills
use super::SkillError;

pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Format a recovery guide for the given error
    pub fn format_recovery_guide(error: &SkillError) -> String {
        match error {
            SkillError::NotFound => {
                "💡 Recovery suggestions:\n\
                 • List available skills: q skills list\n\
                 • Check skill name spelling\n\
                 • Create a new skill: q skills create <name> --from-template command"
                    .to_string()
            }
            SkillError::InvalidInput(msg) => {
                format!(
                    "💡 Recovery suggestions:\n\
                     • Check JSON syntax: {}\n\
                     • Use valid JSON format: {{\"key\": \"value\"}}\n\
                     • Get skill info: q skills info <name>",
                    msg
                )
            }
            SkillError::InvalidConfiguration(msg) => {
                format!(
                    "💡 Recovery suggestions:\n\
                     • Validate skill file: q skills validate <file>\n\
                     • Check required fields: name, description, version, type\n\
                     • Error: {}",
                    msg
                )
            }
            SkillError::ExecutionFailed(msg) => {
                format!(
                    "💡 Recovery suggestions:\n\
                     • Check skill command/script is valid\n\
                     • Verify parameters are correct\n\
                     • Error: {}",
                    msg
                )
            }
            SkillError::Timeout(seconds) => {
                format!(
                    "💡 Recovery suggestions:\n\
                     • Skill timed out after {} seconds\n\
                     • Check if command is hanging\n\
                     • Consider optimizing the skill",
                    seconds
                )
            }
            SkillError::ResourceLimit(msg) => {
                format!(
                    "💡 Recovery suggestions:\n\
                     • Resource limit exceeded: {}\n\
                     • Reduce resource usage in skill\n\
                     • Check system resources",
                    msg
                )
            }
            SkillError::Io(e) => {
                format!(
                    "💡 Recovery suggestions:\n\
                     • IO error: {}\n\
                     • Check file permissions\n\
                     • Verify file paths exist",
                    e
                )
            }
            SkillError::Serialization(e) => {
                format!(
                    "💡 Recovery suggestions:\n\
                     • JSON error: {}\n\
                     • Check JSON syntax\n\
                     • Validate with: q skills validate <file>",
                    e
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_recovery() {
        let error = SkillError::NotFound;
        let guide = ErrorRecovery::format_recovery_guide(&error);
        assert!(guide.contains("q skills list"));
        assert!(guide.contains("Recovery suggestions"));
    }

    #[test]
    fn test_invalid_input_recovery() {
        let error = SkillError::InvalidInput("bad json".to_string());
        let guide = ErrorRecovery::format_recovery_guide(&error);
        assert!(guide.contains("JSON syntax"));
        assert!(guide.contains("bad json"));
    }

    #[test]
    fn test_execution_failed_recovery() {
        let error = SkillError::ExecutionFailed("command not found".to_string());
        let guide = ErrorRecovery::format_recovery_guide(&error);
        assert!(guide.contains("command not found"));
        assert!(guide.contains("parameters"));
    }
}

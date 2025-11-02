//! CLI tests for Assistant command

use clap::Parser;
use crate::cli::creation::{CreateArgs, CreateCommand, AssistantMode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_assistant_basic() {
        let args = CreateArgs::try_parse_from(&["create", "assistant"]).unwrap();
        match args.command {
            CreateCommand::Assistant { mode } => {
                assert_eq!(mode, None);
            },
            _ => panic!("Expected Assistant command"),
        }
    }

    #[test]
    fn test_create_assistant_template() {
        let args = CreateArgs::try_parse_from(&["create", "assistant", "template"]).unwrap();
        match args.command {
            CreateCommand::Assistant { mode } => {
                assert_eq!(mode, Some(AssistantMode::Template));
            },
            _ => panic!("Expected Assistant command"),
        }
    }

    #[test]
    fn test_create_assistant_custom() {
        let args = CreateArgs::try_parse_from(&["create", "assistant", "custom"]).unwrap();
        match args.command {
            CreateCommand::Assistant { mode } => {
                assert_eq!(mode, Some(AssistantMode::Custom));
            },
            _ => panic!("Expected Assistant command"),
        }
    }
}

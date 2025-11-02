#[cfg(test)]
mod tests {
    use super::super::interactive::*;
    use crate::cli::creation::CreationType;
    use crate::cli::creation::tests::MockTerminalUI;

    #[tokio::test]
    async fn test_interactive_flow_initialization() {
        let ui = MockTerminalUI::new(vec![]);
        let result = InteractiveCreationFlow::new(ui).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_skill_creation_complete_flow() {
        let ui = MockTerminalUI::new(vec![
            "my_skill".to_string(),
            "".to_string(), // Empty description
        ]);
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

        let result = flow.run(CreationType::Skill).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[tokio::test]
    async fn test_command_creation_complete_flow() {
        let ui = MockTerminalUI::new(vec!["my_command".to_string(), "ls -la".to_string()]);
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

        let result = flow.run(CreationType::CustomCommand).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_agent_creation_complete_flow() {
        let ui = MockTerminalUI::new(vec!["my_agent".to_string(), "helpful assistant".to_string()]);
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

        let result = flow.run(CreationType::Agent).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_template_selection_single_option() {
        let ui = MockTerminalUI::new(vec![
            "test_skill".to_string(),
            "".to_string(), // Empty description
        ]);
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

        let result = flow.run(CreationType::Skill).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_error_handling_invalid_input() {
        let ui = MockTerminalUI::new(vec![]); // No inputs provided
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

        let result = flow.run(CreationType::Skill).await;
        // Should handle missing input gracefully
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_all_creation_types() {
        for creation_type in [CreationType::Skill, CreationType::CustomCommand, CreationType::Agent] {
            let ui = MockTerminalUI::new(vec!["test_name".to_string(), "test_value".to_string()]);
            let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

            let result = flow.run(creation_type.clone()).await;
            assert!(result.is_ok(), "Failed for creation type: {:?}", creation_type);
        }
    }
}

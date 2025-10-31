#[cfg(test)]
mod chat_integration_tests {
    use crate::cli::skills::{SkillRegistry, SkillError};
    use serde_json::json;

    struct MockChatSession {
        registry: SkillRegistry,
    }

    impl MockChatSession {
        fn new() -> Self {
            Self {
                registry: SkillRegistry::with_builtins(),
            }
        }

        async fn process_input(&mut self, input: &str) -> Result<String, SkillError> {
            // Parse @skill_name syntax
            if let Some(skill_invocation) = self.parse_skill_invocation(input) {
                let result = self.registry.execute_skill(
                    &skill_invocation.skill_name,
                    skill_invocation.params,
                ).await?;
                Ok(result.output)
            } else {
                Ok("Regular chat response".to_string())
            }
        }

        fn parse_skill_invocation(&self, input: &str) -> Option<SkillInvocation> {
            if !input.starts_with('@') {
                return None;
            }

            let parts: Vec<&str> = input[1..].split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }

            let skill_name = parts[0].to_string();
            let args = &parts[1..];

            // Simple parsing for test purposes
            let params = match skill_name.as_str() {
                "calculator" => {
                    if args.len() >= 3 {
                        json!({
                            "op": args[0],
                            "a": args[1].parse::<f64>().unwrap_or(0.0),
                            "b": args[2].parse::<f64>().unwrap_or(0.0)
                        })
                    } else {
                        json!({})
                    }
                },
                _ => json!({}),
            };

            Some(SkillInvocation {
                skill_name,
                params,
            })
        }
    }

    struct SkillInvocation {
        skill_name: String,
        params: serde_json::Value,
    }

    #[tokio::test]
    async fn test_skill_invocation_from_chat() {
        let mut chat = MockChatSession::new();
        
        let response = chat.process_input("@calculator add 2 3").await.unwrap();
        assert_eq!(response, "5");
    }

    #[tokio::test]
    async fn test_skill_invocation_with_different_operations() {
        let mut chat = MockChatSession::new();
        
        let add_response = chat.process_input("@calculator add 10 5").await.unwrap();
        assert_eq!(add_response, "15");
        
        let subtract_response = chat.process_input("@calculator subtract 10 5").await.unwrap();
        assert_eq!(subtract_response, "5");
        
        let multiply_response = chat.process_input("@calculator multiply 10 5").await.unwrap();
        assert_eq!(multiply_response, "50");
        
        let divide_response = chat.process_input("@calculator divide 10 5").await.unwrap();
        assert_eq!(divide_response, "2");
    }

    #[tokio::test]
    async fn test_skill_invocation_error_handling() {
        let mut chat = MockChatSession::new();
        
        // Test division by zero
        let result = chat.process_input("@calculator divide 10 0").await;
        assert!(result.is_err());
        
        // Test unknown skill
        let result = chat.process_input("@nonexistent_skill").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_regular_chat_input_not_affected() {
        let mut chat = MockChatSession::new();
        
        let response = chat.process_input("Hello, how are you?").await.unwrap();
        assert_eq!(response, "Regular chat response");
        
        let response = chat.process_input("What is 2 + 3?").await.unwrap();
        assert_eq!(response, "Regular chat response");
    }

    #[tokio::test]
    async fn test_skill_invocation_parsing() {
        let chat = MockChatSession::new();
        
        // Valid skill invocation
        let invocation = chat.parse_skill_invocation("@calculator add 2 3");
        assert!(invocation.is_some());
        let inv = invocation.unwrap();
        assert_eq!(inv.skill_name, "calculator");
        assert_eq!(inv.params["op"], "add");
        assert_eq!(inv.params["a"], 2.0);
        assert_eq!(inv.params["b"], 3.0);
        
        // Invalid skill invocation (no @)
        let invocation = chat.parse_skill_invocation("calculator add 2 3");
        assert!(invocation.is_none());
        
        // Empty skill invocation
        let invocation = chat.parse_skill_invocation("@");
        assert!(invocation.is_none());
    }

    #[tokio::test]
    async fn test_skill_list_available_skills() {
        let chat = MockChatSession::new();
        let skills = chat.registry.list();
        
        assert!(!skills.is_empty());
        assert!(skills.iter().any(|s| s.name() == "calculator"));
    }

    #[tokio::test]
    async fn test_skill_help_information() {
        let chat = MockChatSession::new();
        
        if let Some(calculator) = chat.registry.get("calculator") {
            assert_eq!(calculator.name(), "calculator");
            assert!(!calculator.description().is_empty());
            assert!(calculator.supports_interactive());
        } else {
            panic!("Calculator skill should be available");
        }
    }
}

#[cfg(test)]
mod skill_interface_tests {
    use serde_json::json;
    use tokio::time::{Duration, timeout};

    use crate::cli::skills::{Skill, SkillError, SkillResult, SkillUI, UIElement};

    struct TestSkill {
        name: String,
        state: std::sync::Arc<std::sync::Mutex<serde_json::Value>>,
    }

    impl TestSkill {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                state: std::sync::Arc::new(std::sync::Mutex::new(json!({}))),
            }
        }
    }

    #[async_trait::async_trait]
    impl Skill for TestSkill {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Test skill for unit testing"
        }

        async fn execute(&self, params: serde_json::Value) -> Result<SkillResult, SkillError> {
            match self.name.as_str() {
                "calculator" => {
                    let a = params["a"].as_i64().unwrap_or(0);
                    let b = params["b"].as_i64().unwrap_or(0);
                    let op = params["op"].as_str().unwrap_or("add");

                    let result = match op {
                        "add" => a + b,
                        "subtract" => a - b,
                        _ => 0,
                    };

                    Ok(SkillResult {
                        output: result.to_string(),
                        ui_updates: None,
                        state_changes: None,
                    })
                },
                "counter" => {
                    let mut state = self.state.lock().unwrap();
                    let action = params["action"].as_str().unwrap_or("get");

                    match action {
                        "increment" => {
                            let current = state["count"].as_i64().unwrap_or(0);
                            state["count"] = json!(current + 1);
                            Ok(SkillResult {
                                output: "incremented".to_string(),
                                ui_updates: None,
                                state_changes: Some(state.clone()),
                            })
                        },
                        "get" => {
                            let count = state["count"].as_i64().unwrap_or(0);
                            Ok(SkillResult {
                                output: count.to_string(),
                                ui_updates: None,
                                state_changes: None,
                            })
                        },
                        _ => Err(SkillError::InvalidInput("Unknown action".to_string())),
                    }
                },
                "failing_skill" => Err(SkillError::ExecutionFailed("Intentional failure".to_string())),
                "slow_skill" => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    Ok(SkillResult {
                        output: "slow result".to_string(),
                        ui_updates: None,
                        state_changes: None,
                    })
                },
                _ => Err(SkillError::NotFound),
            }
        }

        async fn render_ui(&self) -> Result<SkillUI, SkillError> {
            Ok(SkillUI {
                elements: vec![UIElement::Text("Test UI".to_string())],
                interactive: self.supports_interactive(),
            })
        }

        fn supports_interactive(&self) -> bool {
            self.name == "file_browser"
        }
    }

    #[tokio::test]
    async fn test_skill_basic_execution() {
        let skill = TestSkill::new("calculator");
        let params = json!({"a": 2, "b": 3, "op": "add"});

        let result = skill.execute(params).await.unwrap();
        assert_eq!(result.output, "5");
    }

    #[tokio::test]
    async fn test_skill_maintains_state() {
        let skill = TestSkill::new("counter");

        // Increment counter
        let result1 = skill.execute(json!({"action": "increment"})).await.unwrap();
        assert_eq!(result1.output, "incremented");
        assert!(result1.state_changes.is_some());

        // Get counter value
        let result2 = skill.execute(json!({"action": "get"})).await.unwrap();
        assert_eq!(result2.output, "1");
    }

    #[tokio::test]
    async fn test_skill_error_handling() {
        let skill = TestSkill::new("failing_skill");

        let result = skill.execute(json!({})).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SkillError::ExecutionFailed(msg) => assert_eq!(msg, "Intentional failure"),
            _ => panic!("Expected ExecutionFailed error"),
        }
    }

    #[tokio::test]
    async fn test_skill_timeout_handling() {
        let skill = TestSkill::new("slow_skill");

        let result = timeout(Duration::from_secs(1), skill.execute(json!({}))).await;
        assert!(result.is_err()); // Should timeout
    }

    #[tokio::test]
    async fn test_skill_ui_rendering() {
        let skill = TestSkill::new("file_browser");

        let ui = skill.render_ui().await.unwrap();
        assert!(!ui.elements.is_empty());
        assert!(ui.interactive);
    }

    #[tokio::test]
    async fn test_skill_metadata() {
        let skill = TestSkill::new("calculator");

        assert_eq!(skill.name(), "calculator");
        assert!(!skill.description().is_empty());
        assert!(!skill.supports_interactive());
    }
}

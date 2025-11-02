#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::cli::skills::builtin::calculator::Calculator;
    use crate::cli::skills::{
        Skill,
        SkillError,
        SkillRegistry,
    };

    #[tokio::test]
    async fn test_calculator_basic_operations() {
        let calc = Calculator::new().unwrap();

        // Test addition
        let result = calc.execute(json!({"a": 5, "b": 3, "op": "add"})).await.unwrap();
        assert_eq!(result.output, "8");

        // Test subtraction
        let result = calc.execute(json!({"a": 10, "b": 4, "op": "subtract"})).await.unwrap();
        assert_eq!(result.output, "6");
    }

    #[tokio::test]
    async fn test_calculator_error_cases() {
        let calc = Calculator::new().unwrap();

        // Test division by zero
        let result = calc.execute(json!({"a": 10, "b": 0, "op": "divide"})).await;
        assert!(result.is_err());

        // Test missing parameters
        let result = calc.execute(json!({"a": 5})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_skill_registry() {
        let registry = SkillRegistry::with_builtins();

        // Test calculator is registered
        assert!(registry.get("calculator").is_some());
        assert!(registry.get("calc").is_some());

        // Test execution through registry
        let result = registry
            .execute_skill("calculator", json!({"a": 2, "b": 3, "op": "add"}))
            .await
            .unwrap();
        assert_eq!(result.output, "5");
    }

    #[test]
    fn test_error_handling() {
        let error = SkillError::NotFound;
        assert_eq!(error.to_string(), "Skill not found");
    }
}

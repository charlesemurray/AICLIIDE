#[cfg(test)]
mod quality_validator_tests {
    use crate::cli::creation::prompt_system::template_manager::{
        MultiDimensionalValidator,
        QualityValidator,
    };

    #[test]
    fn test_quality_validator_checks_role_clarity() {
        let validator = MultiDimensionalValidator::new();
        let clear_role = "You are an expert code reviewer specializing in Rust, with deep knowledge of memory safety, concurrency patterns, and idiomatic Rust practices.";
        let vague_role = "You help.";

        let clear_score = validator.validate(clear_role);
        let vague_score = validator.validate(vague_role);

        assert!(
            clear_score.overall_score > vague_score.overall_score,
            "Clear role score {} should be > vague role score {}",
            clear_score.overall_score,
            vague_score.overall_score
        );
        assert!(
            clear_score.component_scores.contains_key("role_clarity"),
            "Should have role_clarity component score"
        );
        assert!(
            clear_score.component_scores["role_clarity"] > 0.7,
            "Clear role should score > 0.7, got {}",
            clear_score.component_scores["role_clarity"]
        );
        assert!(
            vague_score.component_scores["role_clarity"] < 0.3,
            "Vague role should score < 0.3, got {}",
            vague_score.component_scores["role_clarity"]
        );
    }

    #[test]
    fn test_role_clarity_considers_length() {
        let validator = MultiDimensionalValidator::new();
        let detailed = "You are a senior software architect with 15 years of experience in distributed systems, microservices architecture, and cloud-native applications.";
        let minimal = "You are helpful.";

        let detailed_score = validator.validate(detailed);
        let minimal_score = validator.validate(minimal);

        assert!(detailed_score.component_scores["role_clarity"] > minimal_score.component_scores["role_clarity"]);
    }

    #[test]
    fn test_role_clarity_considers_specificity() {
        let validator = MultiDimensionalValidator::new();
        let specific = "You are an expert in Rust async programming, tokio runtime, and concurrent data structures.";
        let generic = "You are good at programming.";

        let specific_score = validator.validate(specific);
        let generic_score = validator.validate(generic);

        assert!(specific_score.component_scores["role_clarity"] > generic_score.component_scores["role_clarity"]);
    }
}

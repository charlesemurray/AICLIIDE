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

    #[test]
    fn test_quality_validator_checks_capabilities() {
        let validator = MultiDimensionalValidator::new();
        let detailed = "Capabilities:\n- Analyze code for bugs\n- Find security vulnerabilities\n- Suggest performance improvements\n- Review architecture decisions";
        let minimal = "Capabilities:\n- Help";
        
        let detailed_score = validator.validate(detailed);
        let minimal_score = validator.validate(minimal);
        
        assert!(detailed_score.component_scores.contains_key("capability_completeness"),
            "Should have capability_completeness score");
        assert!(detailed_score.component_scores["capability_completeness"] > 
                minimal_score.component_scores["capability_completeness"],
            "Detailed capabilities {} should score higher than minimal {}",
            detailed_score.component_scores["capability_completeness"],
            minimal_score.component_scores["capability_completeness"]);
    }

    #[test]
    fn test_capability_completeness_counts_items() {
        let validator = MultiDimensionalValidator::new();
        let many = "Capabilities:\n- Item 1\n- Item 2\n- Item 3\n- Item 4\n- Item 5";
        let few = "Capabilities:\n- Item 1";
        
        let many_score = validator.validate(many);
        let few_score = validator.validate(few);
        
        assert!(many_score.component_scores["capability_completeness"] > 0.7);
        assert!(few_score.component_scores["capability_completeness"] < 0.4);
    }

    #[test]
    fn test_capability_completeness_requires_specificity() {
        let validator = MultiDimensionalValidator::new();
        let specific = "Capabilities:\n- Analyze code for memory leaks\n- Detect race conditions\n- Validate error handling";
        let vague = "Capabilities:\n- Do things\n- Help out\n- Be useful";
        
        let specific_score = validator.validate(specific);
        let vague_score = validator.validate(vague);
        
        assert!(specific_score.component_scores["capability_completeness"] > 
                vague_score.component_scores["capability_completeness"]);
    }

    #[test]
    fn test_quality_validator_checks_constraints() {
        let validator = MultiDimensionalValidator::new();
        let with_constraints = "Constraints:\n- Be concise\n- Cite sources\n- Avoid speculation";
        let without = "Do your best.";
        
        let with_score = validator.validate(with_constraints);
        let without_score = validator.validate(without);
        
        assert!(with_score.component_scores.contains_key("constraint_clarity"),
            "Should have constraint_clarity score");
        assert!(with_score.component_scores["constraint_clarity"] > 
                without_score.component_scores["constraint_clarity"],
            "With constraints {} should score higher than without {}",
            with_score.component_scores["constraint_clarity"],
            without_score.component_scores["constraint_clarity"]);
    }

    #[test]
    fn test_constraint_clarity_counts_items() {
        let validator = MultiDimensionalValidator::new();
        let many = "Constraints:\n- Rule 1\n- Rule 2\n- Rule 3\n- Rule 4";
        let few = "Constraints:\n- Rule 1";
        
        let many_score = validator.validate(many);
        let few_score = validator.validate(few);
        
        assert!(many_score.component_scores["constraint_clarity"] > 0.6);
        assert!(few_score.component_scores["constraint_clarity"] < 0.4);
    }

    #[test]
    fn test_constraint_clarity_requires_specificity() {
        let validator = MultiDimensionalValidator::new();
        let specific = "Constraints:\n- Limit responses to 500 words\n- Always cite sources\n- Never speculate";
        let vague = "Constraints:\n- Be good\n- Try hard";
        
        let specific_score = validator.validate(specific);
        let vague_score = validator.validate(vague);
        
        assert!(specific_score.component_scores["constraint_clarity"] > 
                vague_score.component_scores["constraint_clarity"]);
    }

    #[test]
    fn test_quality_validator_checks_examples() {
        let validator = MultiDimensionalValidator::new();
        let with_examples = "Examples:\nInput: Review this code\nOutput: Here's my analysis...";
        let without = "No examples provided.";
        
        let with_score = validator.validate(with_examples);
        let without_score = validator.validate(without);
        
        assert!(with_score.component_scores.contains_key("example_quality"),
            "Should have example_quality score");
        assert!(with_score.component_scores["example_quality"] > 
                without_score.component_scores["example_quality"],
            "With examples {} should score higher than without {}",
            with_score.component_scores["example_quality"],
            without_score.component_scores["example_quality"]);
    }

    #[test]
    fn test_example_quality_requires_input_output_pairs() {
        let validator = MultiDimensionalValidator::new();
        let complete = "Examples:\nInput: test\nOutput: result\nInput: test2\nOutput: result2";
        let incomplete = "Examples:\nSome text here";
        
        let complete_score = validator.validate(complete);
        let incomplete_score = validator.validate(incomplete);
        
        assert!(complete_score.component_scores["example_quality"] > 0.7);
        assert!(incomplete_score.component_scores["example_quality"] < 0.3);
    }

    #[test]
    fn test_example_quality_counts_pairs() {
        let validator = MultiDimensionalValidator::new();
        let many = "Examples:\nInput: a\nOutput: b\nInput: c\nOutput: d\nInput: e\nOutput: f";
        let few = "Examples:\nInput: a\nOutput: b";
        
        let many_score = validator.validate(many);
        let few_score = validator.validate(few);
        
        assert!(many_score.component_scores["example_quality"] > 
                few_score.component_scores["example_quality"]);
    }
}

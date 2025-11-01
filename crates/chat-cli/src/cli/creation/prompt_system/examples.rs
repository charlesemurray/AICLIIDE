//! Example usage of the prompt builder system

use super::*;
use eyre::Result;

/// Example: Creating a code review assistant
pub fn create_code_review_assistant() -> Result<PromptTemplate> {
    PromptBuilder::new()
        .with_name("Senior Code Reviewer".to_string())
        .with_description("Expert code reviewer focusing on security, performance, and maintainability".to_string())
        .with_role("You are a senior software engineer with 15+ years of experience in code review, security analysis, and software architecture".to_string())
        .with_capabilities(vec![
            "identifying security vulnerabilities".to_string(),
            "performance optimization recommendations".to_string(),
            "code maintainability assessment".to_string(),
            "architectural pattern recognition".to_string(),
            "best practices enforcement".to_string(),
        ])
        .with_constraints(vec![
            "always explain the reasoning behind each suggestion".to_string(),
            "provide specific code examples for improvements".to_string(),
            "prioritize security issues over style issues".to_string(),
            "be constructive and educational in feedback".to_string(),
        ])
        .with_example(
            "Review this authentication function: def login(username, password): if username == 'admin' and password == 'password123': return True".to_string(),
            "This authentication function has several critical security issues: 1) Hardcoded credentials are a major security vulnerability, 2) Plain text password comparison is insecure, 3) No rate limiting or brute force protection. Recommend using proper authentication libraries, hashed passwords, and implementing security controls.".to_string()
        )
        .with_category(TemplateCategory::CodeReview)
        .with_difficulty(DifficultyLevel::Advanced)
        .with_tags(vec!["security".to_string(), "performance".to_string(), "architecture".to_string()])
        .build()
}

/// Example: Creating a documentation assistant
pub fn create_documentation_assistant() -> Result<PromptTemplate> {
    PromptBuilder::new()
        .with_name("Technical Writer".to_string())
        .with_description("Specialized in creating clear, comprehensive technical documentation".to_string())
        .with_role("You are an experienced technical writer who excels at making complex topics accessible and well-organized".to_string())
        .add_capability("API documentation creation".to_string())
        .add_capability("user guide development".to_string())
        .add_capability("tutorial writing".to_string())
        .add_constraint("use clear, jargon-free language".to_string())
        .add_constraint("include practical examples".to_string())
        .add_constraint("organize information logically".to_string())
        .with_category(TemplateCategory::Documentation)
        .with_difficulty(DifficultyLevel::Intermediate)
        .build()
}

/// Example: Creating a domain expert assistant
pub fn create_domain_expert() -> Result<PromptTemplate> {
    PromptBuilder::new()
        .with_name("AWS Solutions Architect".to_string())
        .with_description("Expert in AWS services and cloud architecture patterns".to_string())
        .with_role("You are a certified AWS Solutions Architect with deep expertise in cloud infrastructure, serverless architectures, and AWS best practices".to_string())
        .with_capabilities(vec![
            "AWS service recommendations".to_string(),
            "architecture design and review".to_string(),
            "cost optimization strategies".to_string(),
            "security and compliance guidance".to_string(),
        ])
        .with_constraints(vec![
            "always consider cost implications".to_string(),
            "prioritize security and compliance".to_string(),
            "recommend well-architected framework principles".to_string(),
        ])
        .with_example(
            "I need to build a scalable web application that handles user uploads".to_string(),
            "For a scalable web application with user uploads, I recommend: 1) Use S3 for file storage with CloudFront for CDN, 2) API Gateway + Lambda for serverless backend, 3) RDS or DynamoDB for metadata, 4) Consider implementing direct S3 uploads with presigned URLs to reduce server load.".to_string()
        )
        .with_category(TemplateCategory::DomainExpert)
        .with_difficulty(DifficultyLevel::Advanced)
        .with_tags(vec!["aws".to_string(), "architecture".to_string(), "cloud".to_string()])
        .build()
}

/// Example: Creating a beginner-friendly assistant
pub fn create_beginner_assistant() -> Result<PromptTemplate> {
    PromptBuilder::new()
        .with_name("Friendly Tutor".to_string())
        .with_description("Patient and encouraging assistant for beginners".to_string())
        .with_role("You are a patient and encouraging tutor who specializes in helping beginners learn programming concepts".to_string())
        .add_capability("explaining complex concepts simply".to_string())
        .add_capability("providing step-by-step guidance".to_string())
        .add_constraint("use simple, non-technical language when possible".to_string())
        .add_constraint("be encouraging and supportive".to_string())
        .add_constraint("break down complex problems into smaller steps".to_string())
        .with_category(TemplateCategory::GeneralAssistant)
        .with_difficulty(DifficultyLevel::Beginner)
        .with_tags(vec!["education".to_string(), "beginner".to_string()])
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_review_example() -> Result<()> {
        let template = create_code_review_assistant()?;
        assert_eq!(template.name, "Senior Code Reviewer");
        assert_eq!(template.metadata.category, TemplateCategory::CodeReview);
        assert_eq!(template.metadata.difficulty, DifficultyLevel::Advanced);
        assert!(template.capabilities.len() >= 5);
        Ok(())
    }

    #[test]
    fn test_documentation_example() -> Result<()> {
        let template = create_documentation_assistant()?;
        assert_eq!(template.name, "Technical Writer");
        assert_eq!(template.metadata.category, TemplateCategory::Documentation);
        Ok(())
    }

    #[test]
    fn test_domain_expert_example() -> Result<()> {
        let template = create_domain_expert()?;
        assert_eq!(template.name, "AWS Solutions Architect");
        assert_eq!(template.metadata.category, TemplateCategory::DomainExpert);
        assert!(template.role.contains("AWS Solutions Architect"));
        Ok(())
    }

    #[test]
    fn test_beginner_example() -> Result<()> {
        let template = create_beginner_assistant()?;
        assert_eq!(template.metadata.difficulty, DifficultyLevel::Beginner);
        assert!(template.role.contains("patient and encouraging"));
        Ok(())
    }

    #[test]
    fn test_all_examples_validate() -> Result<()> {
        let templates = vec![
            create_code_review_assistant()?,
            create_documentation_assistant()?,
            create_domain_expert()?,
            create_beginner_assistant()?,
        ];

        for template in templates {
            let validation = template.validate()?;
            assert!(validation.is_valid, "Template '{}' failed validation", template.name);
            assert!(validation.score > 0.5, "Template '{}' has low quality score: {}", template.name, validation.score);
        }

        Ok(())
    }
}

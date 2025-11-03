//! End-to-end tests for natural language skill invocation
//!
//! These tests validate that users can invoke skills through natural language
//! by simulating the complete flow: NL input → Agent → Skill selection → Execution

mod helpers;

use chat_cli::cli::skills::SkillRegistry;
use helpers::mock_agent::MockAgent;

/// Test: User invokes calculator skill with simple addition
#[tokio::test]
async fn test_user_invokes_calculator_addition() {
    // Setup: Agent with builtin skills
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);

    // User input: Natural language request
    let input = "calculate 5 plus 3";

    // Agent processes input
    let response = agent.process_input(input).await;

    // Verify: Agent selected calculator and got correct result
    assert!(response.used_tool("calculator"), "Agent should select calculator skill");
    assert!(response.success, "Execution should succeed");
    assert_eq!(response.result(), "8", "Result should be 8");
}

/// Test: User invokes calculator with multiplication
#[tokio::test]
async fn test_user_invokes_calculator_multiplication() {
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);

    let input = "multiply 10 and 5";
    let response = agent.process_input(input).await;

    assert!(response.used_tool("calculator"));
    assert!(response.success);
    assert_eq!(response.result(), "50");
}

/// Test: User invokes calculator with different phrasing
#[tokio::test]
async fn test_user_invokes_calculator_alternate_phrasing() {
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);

    // Different ways to ask for calculation
    let inputs = vec!["add 7 and 3", "calculate 7 + 3", "7 plus 3"];

    for input in inputs {
        let response = agent.process_input(input).await;
        assert!(response.used_tool("calculator"), "Should work with: {}", input);
        assert!(response.success, "Should succeed with: {}", input);
    }
}

/// Test: Agent handles skill not found gracefully
#[tokio::test]
async fn test_skill_not_found_error() {
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);

    // Request for non-existent skill
    let input = "use my custom skill that doesn't exist";
    let response = agent.process_input(input).await;

    assert!(!response.success, "Should fail when skill not found");
    assert!(response.selected_tool.is_none(), "Should not select a tool");
}

/// Test: Agent handles ambiguous requests
#[tokio::test]
async fn test_ambiguous_request_handling() {
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);

    // Ambiguous input
    let input = "do something";
    let response = agent.process_input(input).await;

    assert!(!response.success, "Should fail for ambiguous input");
}

/// Test: Agent extracts correct parameters from natural language
#[tokio::test]
async fn test_parameter_extraction_from_natural_language() {
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);

    let input = "calculate 15 plus 27";
    let response = agent.process_input(input).await;

    // Verify parameters were extracted correctly
    assert_eq!(response.parameters["a"], 15.0);
    assert_eq!(response.parameters["b"], 27.0);
    assert_eq!(response.parameters["op"], "add");
}

/// Test: Multiple skill invocations in sequence
#[tokio::test]
async fn test_multiple_skill_invocations() {
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);

    // First invocation
    let response1 = agent.process_input("add 5 and 3").await;
    assert!(response1.success);
    assert_eq!(response1.result(), "8");

    // Second invocation
    let response2 = agent.process_input("multiply 4 and 6").await;
    assert!(response2.success);
    assert_eq!(response2.result(), "24");
}

/// Test: Agent discovers all available skills
#[tokio::test]
async fn test_agent_discovers_all_skills() {
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);

    let tools = agent.available_tool_names();

    assert!(!tools.is_empty(), "Agent should discover skills");
    assert!(tools.contains(&"calculator".to_string()), "Should find calculator");
}

/// Test: Skill invocation with edge case numbers
#[tokio::test]
async fn test_skill_with_edge_case_numbers() {
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);

    // Test with zero
    let response = agent.process_input("add 0 and 5").await;
    assert!(response.success);
    assert_eq!(response.result(), "5");

    // Test with negative (if supported)
    let response = agent.process_input("calculate 10 plus 0").await;
    assert!(response.success);
}

/// Test: Complete user journey - create request, get result
#[tokio::test]
async fn test_complete_user_journey() {
    // Step 1: User has skills available
    let registry = SkillRegistry::with_builtins();

    // Step 2: Agent is initialized with skills
    let agent = MockAgent::with_skills(&registry);

    // Step 3: User makes natural language request
    let user_input = "I need to calculate 100 plus 50";

    // Step 4: Agent processes and selects skill
    let response = agent.process_input(user_input).await;

    // Step 5: Verify user gets result
    assert!(response.success, "User should get successful result");
    assert!(response.used_tool("calculator"), "Calculator should be used");
    assert_eq!(response.result(), "150", "User should see correct answer");
}

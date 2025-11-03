//! Evaluation framework for memory recall quality

use cortex_memory::{CortexMemory, MemoryConfig};
use std::collections::HashMap;
use tempfile::TempDir;

/// Test case for evaluation
struct EvaluationCase {
    query: String,
    expected_keywords: Vec<String>,
    min_score: f32,
}

/// Evaluation metrics
#[derive(Debug)]
struct EvaluationMetrics {
    precision_at_5: f32,
    recall_at_5: f32,
    avg_score: f32,
    cases_passed: usize,
    cases_total: usize,
}

/// Create test dataset with known interactions
fn create_test_dataset() -> Vec<(String, String, String)> {
    vec![
        // (user_message, assistant_response, session_id)
        (
            "How do I implement authentication in Rust?".to_string(),
            "To implement authentication in Rust, you can use the `jsonwebtoken` crate for JWT tokens...".to_string(),
            "session1".to_string(),
        ),
        (
            "What's the best way to handle errors in Rust?".to_string(),
            "Rust uses the Result type for error handling. Use `?` operator for propagation...".to_string(),
            "session1".to_string(),
        ),
        (
            "How do I connect to a PostgreSQL database?".to_string(),
            "Use the `tokio-postgres` crate. Here's an example: `let (client, connection) = tokio_postgres::connect(...)`".to_string(),
            "session2".to_string(),
        ),
        (
            "Explain async/await in Rust".to_string(),
            "Async/await in Rust allows non-blocking operations. Use `async fn` and `.await` on futures...".to_string(),
            "session2".to_string(),
        ),
        (
            "How do I parse JSON in Rust?".to_string(),
            "Use the `serde_json` crate. Derive Serialize/Deserialize on your structs...".to_string(),
            "session3".to_string(),
        ),
    ]
}

/// Create evaluation test cases
fn create_evaluation_cases() -> Vec<EvaluationCase> {
    vec![
        EvaluationCase {
            query: "authentication JWT".to_string(),
            expected_keywords: vec!["authentication".to_string(), "JWT".to_string(), "jsonwebtoken".to_string()],
            min_score: 0.6,
        },
        EvaluationCase {
            query: "error handling Result".to_string(),
            expected_keywords: vec!["error".to_string(), "Result".to_string()],
            min_score: 0.6,
        },
        EvaluationCase {
            query: "database PostgreSQL".to_string(),
            expected_keywords: vec!["PostgreSQL".to_string(), "database".to_string()],
            min_score: 0.6,
        },
        EvaluationCase {
            query: "async await futures".to_string(),
            expected_keywords: vec!["async".to_string(), "await".to_string()],
            min_score: 0.6,
        },
        EvaluationCase {
            query: "JSON parsing serde".to_string(),
            expected_keywords: vec!["JSON".to_string(), "serde".to_string()],
            min_score: 0.6,
        },
    ]
}

/// Evaluate recall quality
fn evaluate_recall(memory: &mut CortexMemory, cases: &[EvaluationCase]) -> EvaluationMetrics {
    let mut total_precision = 0.0;
    let mut total_recall = 0.0;
    let mut total_score = 0.0;
    let mut cases_passed = 0;

    for case in cases {
        let results = memory.recall_context(&case.query, 5).unwrap_or_default();
        
        if results.is_empty() {
            continue;
        }

        // Calculate precision: how many results contain expected keywords
        let mut relevant_count = 0;
        let mut score_sum = 0.0;

        for item in &results {
            score_sum += item.score;
            
            // Check if content contains any expected keywords
            let content_lower = item.content.to_lowercase();
            if case.expected_keywords.iter().any(|kw| content_lower.contains(&kw.to_lowercase())) {
                relevant_count += 1;
            }
        }

        let precision = relevant_count as f32 / results.len() as f32;
        let recall = relevant_count as f32 / case.expected_keywords.len() as f32;
        let avg_score = score_sum / results.len() as f32;

        total_precision += precision;
        total_recall += recall;
        total_score += avg_score;

        // Case passes if we found relevant results with good scores
        if relevant_count > 0 && avg_score >= case.min_score {
            cases_passed += 1;
        }
    }

    let cases_total = cases.len();
    
    EvaluationMetrics {
        precision_at_5: total_precision / cases_total as f32,
        recall_at_5: total_recall / cases_total as f32,
        avg_score: total_score / cases_total as f32,
        cases_passed,
        cases_total,
    }
}

#[test]
#[ignore] // Requires embedding model files
fn test_memory_recall_quality() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("eval.db");
    let config = MemoryConfig::default().with_enabled(true);
    let mut memory = CortexMemory::new(&db_path, config).unwrap();

    // Populate with test data
    let dataset = create_test_dataset();
    for (user_msg, assistant_msg, session_id) in dataset {
        let _ = memory.store_interaction(&user_msg, &assistant_msg, &session_id);
    }

    // Run evaluation
    let cases = create_evaluation_cases();
    let metrics = evaluate_recall(&mut memory, &cases);

    println!("\n=== Memory Recall Quality Evaluation ===");
    println!("Precision@5: {:.2}%", metrics.precision_at_5 * 100.0);
    println!("Recall@5: {:.2}%", metrics.recall_at_5 * 100.0);
    println!("Avg Score: {:.3}", metrics.avg_score);
    println!("Cases Passed: {}/{}", metrics.cases_passed, metrics.cases_total);
    println!("Pass Rate: {:.1}%", (metrics.cases_passed as f32 / metrics.cases_total as f32) * 100.0);

    // Quality thresholds
    assert!(
        metrics.precision_at_5 >= 0.6,
        "Precision@5 too low: {:.2} (expected >= 0.6)",
        metrics.precision_at_5
    );
    
    assert!(
        metrics.avg_score >= 0.5,
        "Average score too low: {:.3} (expected >= 0.5)",
        metrics.avg_score
    );

    assert!(
        metrics.cases_passed >= (metrics.cases_total * 3 / 5),
        "Too many cases failed: {}/{} (expected >= 60%)",
        metrics.cases_passed,
        metrics.cases_total
    );

    println!("\n✅ All quality thresholds met!");
}

#[test]
#[ignore] // Requires embedding model files
fn test_session_isolation_quality() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("eval_session.db");
    let config = MemoryConfig::default().with_enabled(true);
    let mut memory = CortexMemory::new(&db_path, config).unwrap();

    // Store data in different sessions
    let _ = memory.store_interaction(
        "How do I use Redis?",
        "Redis is an in-memory data store. Use the redis crate...",
        "session_a",
    );
    
    let _ = memory.store_interaction(
        "How do I use MongoDB?",
        "MongoDB is a document database. Use the mongodb crate...",
        "session_b",
    );

    // Query should find both
    let global_results = memory.recall_context("database", 5).unwrap();
    assert!(global_results.len() >= 2, "Should find memories from both sessions");

    // Session-specific query should filter
    let session_results = memory.recall_by_session("database", "session_a", 5).unwrap();
    
    // Verify session isolation
    for item in &session_results {
        let session_id = item.metadata.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(session_id, "session_a", "Should only return session_a memories");
    }

    println!("\n✅ Session isolation working correctly!");
}

#[test]
#[ignore] // Requires embedding model files
fn test_deduplication_quality() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("eval_dedup.db");
    let config = MemoryConfig::default().with_enabled(true);
    let mut memory = CortexMemory::new(&db_path, config).unwrap();

    // Store same interaction twice
    let id1 = memory.store_interaction(
        "What is Rust?",
        "Rust is a systems programming language...",
        "session1",
    ).unwrap();

    let id2 = memory.store_interaction(
        "What is Rust?",
        "Rust is a systems programming language...",
        "session1",
    ).unwrap();

    // Second store should return empty (duplicate)
    assert!(id2.is_empty(), "Duplicate should not be stored");
    assert!(!id1.is_empty(), "First store should succeed");

    println!("\n✅ Deduplication working correctly!");
}

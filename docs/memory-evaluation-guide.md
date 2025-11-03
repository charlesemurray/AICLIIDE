# Memory System Evaluation Guide

## Overview

The evaluation framework measures memory recall quality using a test dataset with known queries and expected results.

## Running Evaluation

```bash
# Run evaluation tests (requires embedding model)
cargo test -p cortex-memory --test evaluation -- --ignored --nocapture
```

## Evaluation Metrics

### Precision@5
Percentage of recalled memories that are relevant to the query.
- **Target**: ≥ 60%
- **Measures**: Accuracy of results

### Recall@5
Percentage of relevant memories that were found.
- **Target**: ≥ 60%
- **Measures**: Completeness of results

### Average Score
Mean similarity score of recalled memories.
- **Target**: ≥ 0.5
- **Measures**: Relevance quality

### Pass Rate
Percentage of test cases that meet quality thresholds.
- **Target**: ≥ 60%
- **Measures**: Overall system quality

## Test Dataset

The evaluation uses 5 test interactions covering common programming topics:
1. Authentication (JWT, Rust)
2. Error handling (Result type)
3. Database (PostgreSQL)
4. Async/await (Futures)
5. JSON parsing (Serde)

Each test case includes:
- Query to test
- Expected keywords in results
- Minimum relevance score

## Quality Thresholds

Tests will fail if:
- Precision@5 < 60%
- Average score < 0.5
- Pass rate < 60%

## Example Output

```
=== Memory Recall Quality Evaluation ===
Precision@5: 75.00%
Recall@5: 68.00%
Avg Score: 0.723
Cases Passed: 4/5
Pass Rate: 80.0%

✅ All quality thresholds met!
```

## Additional Tests

### Session Isolation
Verifies that session-filtered recall only returns memories from the specified session.

### Deduplication
Verifies that duplicate memories are not stored (similarity > 0.95).

## Running in CI/CD

Add to your CI pipeline:

```yaml
- name: Run Memory Evaluation
  run: cargo test -p cortex-memory --test evaluation -- --ignored
```

## Interpreting Results

### Good Results
- Precision > 70%: Most results are relevant
- Recall > 60%: Finding most relevant memories
- Avg Score > 0.6: High relevance
- Pass Rate > 80%: System working well

### Poor Results
- Precision < 50%: Too many irrelevant results
- Recall < 40%: Missing relevant memories
- Avg Score < 0.4: Low relevance
- Pass Rate < 50%: System needs improvement

### Improvement Actions

**Low Precision:**
- Improve embedding quality
- Add reranking
- Filter low-score results

**Low Recall:**
- Increase search limit
- Improve query expansion
- Check deduplication threshold

**Low Scores:**
- Better embedding model
- Tune similarity thresholds
- Improve content formatting

## Extending Evaluation

To add new test cases:

```rust
EvaluationCase {
    query: "your query here".to_string(),
    expected_keywords: vec!["keyword1".to_string(), "keyword2".to_string()],
    min_score: 0.6,
}
```

Add corresponding data to the test dataset:

```rust
(
    "User question".to_string(),
    "Assistant response".to_string(),
    "session_id".to_string(),
)
```

## Continuous Monitoring

Track metrics over time:
- Run evaluation weekly
- Compare against baseline
- Alert on degradation
- Iterate on improvements

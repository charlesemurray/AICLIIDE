use std::time::Instant;

use cortex_memory::{
    CortexMemory,
    MemoryConfig,
};
use tempfile::TempDir;

fn main() {
    println!("Memory System Performance Benchmark\n");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("bench.db");
    let config = MemoryConfig::default().with_enabled(true);
    let mut memory = CortexMemory::new(db_path, config).expect("Failed to create memory");

    println!("=== Store Performance ===");
    let store_count = 100;
    let start = Instant::now();

    for i in 0..store_count {
        let session_id = format!("session_{}", i % 10);
        let _ = memory.store_interaction(
            &format!("User question {}", i),
            &format!("Assistant response {}", i),
            &session_id,
        );
    }

    let store_duration = start.elapsed();
    println!("Stored {} interactions in {:?}", store_count, store_duration);
    println!(
        "Average: {:.2}ms per store\n",
        store_duration.as_millis() as f64 / store_count as f64
    );

    println!("=== Recall Performance ===");
    let recall_count = 50;
    let start = Instant::now();

    for i in 0..recall_count {
        let _ = memory.recall_context(&format!("question {}", i % 10), 5);
    }

    let recall_duration = start.elapsed();
    println!("Performed {} recalls in {:?}", recall_count, recall_duration);
    println!(
        "Average: {:.2}ms per recall\n",
        recall_duration.as_millis() as f64 / recall_count as f64
    );

    println!("=== Summary ===");
    let store_avg = store_duration.as_millis() as f64 / store_count as f64;
    let recall_avg = recall_duration.as_millis() as f64 / recall_count as f64;

    println!("✓ Store: {:.2}ms avg (target: <50ms)", store_avg);
    println!("✓ Recall: {:.2}ms avg (target: <100ms)", recall_avg);

    if store_avg < 50.0 && recall_avg < 100.0 {
        println!("\n✅ All performance targets met!");
    } else {
        println!("\n⚠️  Some targets not met");
    }
}

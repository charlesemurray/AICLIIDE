//! Test runner and configuration for creation system tests

use super::*;
use std::process::Command;
use std::env;

/// Test configuration and utilities
pub struct TestConfig {
    pub run_integration_tests: bool,
    pub run_ux_tests: bool,
    pub run_compatibility_tests: bool,
    pub test_timeout_seconds: u64,
    pub parallel_execution: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            run_integration_tests: true,
            run_ux_tests: true,
            run_compatibility_tests: true,
            test_timeout_seconds: 30,
            parallel_execution: true,
        }
    }
}

impl TestConfig {
    pub fn from_env() -> Self {
        Self {
            run_integration_tests: env::var("SKIP_INTEGRATION_TESTS").is_err(),
            run_ux_tests: env::var("SKIP_UX_TESTS").is_err(),
            run_compatibility_tests: env::var("SKIP_COMPATIBILITY_TESTS").is_err(),
            test_timeout_seconds: env::var("TEST_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            parallel_execution: env::var("SEQUENTIAL_TESTS").is_err(),
        }
    }
}

/// Test suite runner for comprehensive validation
pub struct TestSuiteRunner {
    config: TestConfig,
}

impl TestSuiteRunner {
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }

    pub async fn run_all_tests(&self) -> Result<TestResults, Box<dyn std::error::Error>> {
        let mut results = TestResults::new();

        println!("🧪 Running Creation System Test Suite");
        println!("=====================================");

        // Unit tests (always run)
        println!("\n📋 Running Unit Tests...");
        let unit_results = self.run_unit_tests().await?;
        results.merge(unit_results);

        // CLI tests (always run)
        println!("\n⌨️  Running CLI Tests...");
        let cli_results = self.run_cli_tests().await?;
        results.merge(cli_results);

        // Integration tests (optional)
        if self.config.run_integration_tests {
            println!("\n🔗 Running Integration Tests...");
            let integration_results = self.run_integration_tests().await?;
            results.merge(integration_results);
        }

        // UX tests (optional)
        if self.config.run_ux_tests {
            println!("\n🎨 Running UX Tests...");
            let ux_results = self.run_ux_tests().await?;
            results.merge(ux_results);
        }

        // Compatibility tests (optional)
        if self.config.run_compatibility_tests {
            println!("\n🔄 Running Compatibility Tests...");
            let compatibility_results = self.run_compatibility_tests().await?;
            results.merge(compatibility_results);
        }

        self.print_summary(&results);
        Ok(results)
    }

    async fn run_unit_tests(&self) -> Result<TestResults, Box<dyn std::error::Error>> {
        let output = Command::new("cargo")
            .args(&["test", "--lib", "creation::tests::unit", "--", "--nocapture"])
            .output()?;

        Ok(TestResults::from_cargo_output(&output))
    }

    async fn run_cli_tests(&self) -> Result<TestResults, Box<dyn std::error::Error>> {
        let output = Command::new("cargo")
            .args(&["test", "--lib", "creation::tests::cli", "--", "--nocapture"])
            .output()?;

        Ok(TestResults::from_cargo_output(&output))
    }

    async fn run_integration_tests(&self) -> Result<TestResults, Box<dyn std::error::Error>> {
        let output = Command::new("cargo")
            .args(&["test", "--lib", "creation::tests::integration", "--", "--nocapture"])
            .output()?;

        Ok(TestResults::from_cargo_output(&output))
    }

    async fn run_ux_tests(&self) -> Result<TestResults, Box<dyn std::error::Error>> {
        let output = Command::new("cargo")
            .args(&["test", "--lib", "creation::tests::ux", "--", "--nocapture"])
            .output()?;

        Ok(TestResults::from_cargo_output(&output))
    }

    async fn run_compatibility_tests(&self) -> Result<TestResults, Box<dyn std::error::Error>> {
        let output = Command::new("cargo")
            .args(&["test", "--lib", "creation::tests::compatibility", "--", "--nocapture"])
            .output()?;

        Ok(TestResults::from_cargo_output(&output))
    }

    fn print_summary(&self, results: &TestResults) {
        println!("\n📊 Test Results Summary");
        println!("=======================");
        println!("✅ Passed: {}", results.passed);
        println!("❌ Failed: {}", results.failed);
        println!("⏭️  Ignored: {}", results.ignored);
        println!("⏱️  Total time: {:.2}s", results.duration_seconds);

        if results.failed > 0 {
            println!("\n❌ FAILED TESTS:");
            for failure in &results.failures {
                println!("  - {}: {}", failure.test_name, failure.error_message);
            }
        }

        if results.passed + results.failed > 0 {
            let success_rate = (results.passed as f64 / (results.passed + results.failed) as f64) * 100.0;
            println!("\n📈 Success Rate: {:.1}%", success_rate);
        }
    }
}

/// Test results aggregation
#[derive(Debug, Default)]
pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub duration_seconds: f64,
    pub failures: Vec<TestFailure>,
}

#[derive(Debug)]
pub struct TestFailure {
    pub test_name: String,
    pub error_message: String,
}

impl TestResults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: TestResults) {
        self.passed += other.passed;
        self.failed += other.failed;
        self.ignored += other.ignored;
        self.duration_seconds += other.duration_seconds;
        self.failures.extend(other.failures);
    }

    pub fn from_cargo_output(output: &std::process::Output) -> Self {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Parse cargo test output
        let mut results = TestResults::new();
        
        // Look for test result summary line
        for line in stdout.lines().chain(stderr.lines()) {
            if line.contains("test result:") {
                // Parse line like: "test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
                if let Some(captures) = regex::Regex::new(r"(\d+) passed; (\d+) failed; (\d+) ignored")
                    .unwrap()
                    .captures(line) 
                {
                    results.passed = captures[1].parse().unwrap_or(0);
                    results.failed = captures[2].parse().unwrap_or(0);
                    results.ignored = captures[3].parse().unwrap_or(0);
                }
            }
            
            // Parse individual test failures
            if line.starts_with("---- ") && line.contains(" stdout ----") {
                let test_name = line.replace("---- ", "").replace(" stdout ----", "");
                results.failures.push(TestFailure {
                    test_name,
                    error_message: "See test output for details".to_string(),
                });
            }
        }
        
        results
    }
}

/// Benchmark runner for performance testing
pub struct BenchmarkRunner;

impl BenchmarkRunner {
    pub async fn run_creation_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
        println!("🏃 Running Creation System Benchmarks");
        println!("=====================================");

        Self::benchmark_command_creation().await?;
        Self::benchmark_skill_creation().await?;
        Self::benchmark_agent_creation().await?;
        Self::benchmark_context_analysis().await?;

        Ok(())
    }

    async fn benchmark_command_creation() -> Result<(), Box<dyn std::error::Error>> {
        use std::time::Instant;
        
        let start = Instant::now();
        let iterations = 100;
        
        for i in 0..iterations {
            let fixtures = TestFixtures::new();
            fixtures.setup_directories();
            
            let mut ui = MockTerminalUI::new(vec![
                format!("echo test{}", i),
                "Test command".to_string(),
                "y".to_string(),
            ]);
            
            let mut flow = CommandCreationFlow::new(&format!("test{}", i), &mut ui);
            let _ = flow.run_single_pass();
        }
        
        let duration = start.elapsed();
        let avg_ms = duration.as_millis() as f64 / iterations as f64;
        
        println!("⚡ Command Creation: {:.2}ms average ({} iterations)", avg_ms, iterations);
        
        Ok(())
    }

    async fn benchmark_skill_creation() -> Result<(), Box<dyn std::error::Error>> {
        use std::time::Instant;
        
        let start = Instant::now();
        let iterations = 50;
        
        for i in 0..iterations {
            let fixtures = TestFixtures::new();
            fixtures.setup_directories();
            
            let mut ui = MockTerminalUI::new(vec![
                format!("python script{}.py", i),
                "Test skill".to_string(),
                "medium".to_string(),
                "y".to_string(),
            ]);
            
            let mut flow = SkillCreationFlow::new(&format!("test{}", i), SkillMode::Guided, &mut ui);
            let _ = flow.run_single_pass();
        }
        
        let duration = start.elapsed();
        let avg_ms = duration.as_millis() as f64 / iterations as f64;
        
        println!("⚡ Skill Creation: {:.2}ms average ({} iterations)", avg_ms, iterations);
        
        Ok(())
    }

    async fn benchmark_agent_creation() -> Result<(), Box<dyn std::error::Error>> {
        use std::time::Instant;
        
        let start = Instant::now();
        let iterations = 20;
        
        for i in 0..iterations {
            let fixtures = TestFixtures::new();
            fixtures.setup_directories();
            
            let mut ui = MockTerminalUI::new(vec![
                format!("You are agent {}", i),
                "Test agent".to_string(),
                "n".to_string(), // no MCP
                "n".to_string(), // no tools
                "y".to_string(), // confirm
            ]);
            
            let mut flow = AgentCreationFlow::new(&format!("test{}", i), AgentMode::Quick, &mut ui);
            let _ = flow.run_single_pass();
        }
        
        let duration = start.elapsed();
        let avg_ms = duration.as_millis() as f64 / iterations as f64;
        
        println!("⚡ Agent Creation: {:.2}ms average ({} iterations)", avg_ms, iterations);
        
        Ok(())
    }

    async fn benchmark_context_analysis() -> Result<(), Box<dyn std::error::Error>> {
        use std::time::Instant;
        
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create complex project structure
        for i in 0..50 {
            std::fs::write(
                fixtures.temp_dir.path().join(format!("file{}.py", i)),
                "print('hello')"
            ).unwrap();
        }
        
        let start = Instant::now();
        let iterations = 100;
        
        for _ in 0..iterations {
            let context = CreationContext::new(fixtures.temp_dir.path()).unwrap();
            let _ = context.suggest_defaults(&CreationType::Skill);
            let _ = context.analyze_project_type();
            let _ = context.get_existing_artifacts();
        }
        
        let duration = start.elapsed();
        let avg_ms = duration.as_millis() as f64 / iterations as f64;
        
        println!("⚡ Context Analysis: {:.2}ms average ({} iterations)", avg_ms, iterations);
        
        Ok(())
    }
}

#[cfg(test)]
mod test_runner_tests {
    use super::*;

    #[tokio::test]
    async fn test_runner_configuration() {
        let config = TestConfig::default();
        assert!(config.run_integration_tests);
        assert!(config.run_ux_tests);
        assert!(config.run_compatibility_tests);
        assert_eq!(config.test_timeout_seconds, 30);
    }

    #[test]
    fn test_results_aggregation() {
        let mut results1 = TestResults {
            passed: 5,
            failed: 1,
            ignored: 0,
            duration_seconds: 1.5,
            failures: vec![TestFailure {
                test_name: "test1".to_string(),
                error_message: "error1".to_string(),
            }],
        };

        let results2 = TestResults {
            passed: 3,
            failed: 2,
            ignored: 1,
            duration_seconds: 2.0,
            failures: vec![TestFailure {
                test_name: "test2".to_string(),
                error_message: "error2".to_string(),
            }],
        };

        results1.merge(results2);

        assert_eq!(results1.passed, 8);
        assert_eq!(results1.failed, 3);
        assert_eq!(results1.ignored, 1);
        assert_eq!(results1.duration_seconds, 3.5);
        assert_eq!(results1.failures.len(), 2);
    }
}

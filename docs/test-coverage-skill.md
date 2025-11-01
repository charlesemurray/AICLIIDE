# Test Coverage Skill Documentation

## Overview

The Test Coverage Skill is a comprehensive testing and quality assurance tool designed specifically for the Amazon Q CLI project. It ensures high test coverage, code quality, and provides detailed analysis of the codebase.

## Features

### 🧪 Comprehensive Testing
- **Unit Tests**: Test individual components and functions
- **Integration Tests**: Test component interactions and workflows
- **Documentation Tests**: Validate code examples in documentation
- **Per-Crate Testing**: Isolated testing of individual crates

### 📊 Coverage Analysis
- **Line Coverage**: Track which lines of code are executed during tests
- **Function Coverage**: Ensure all functions are tested
- **Branch Coverage**: Verify all code paths are exercised
- **HTML Reports**: Visual coverage reports with detailed breakdowns

### 🔍 Code Quality Checks
- **Clippy Linting**: Rust-specific code quality and style checks
- **Format Checking**: Ensure consistent code formatting
- **Static Analysis**: Identify potential issues and improvements

### 📈 Reporting & Analytics
- **Detailed Reports**: Comprehensive analysis with actionable recommendations
- **Trend Analysis**: Track coverage changes over time
- **CI/CD Integration**: Automated testing in GitHub Actions

## Installation & Setup

### Prerequisites
- Rust toolchain (stable and nightly)
- Python 3.6+
- Git

### Install Testing Tools
```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Make scripts executable
chmod +x scripts/test_coverage.py
chmod +x scripts/quick_test.sh
```

## Usage

### Using the Q Skill

1. **Load the skill**:
   ```bash
   q skills load test-coverage-analyzer
   ```

2. **Run comprehensive analysis**:
   ```bash
   q skills run test-coverage-analyzer full-analysis
   ```

3. **Quick testing**:
   ```bash
   q skills run test-coverage-analyzer quick-test
   ```

### Available Commands

| Command | Description |
|---------|-------------|
| `full-analysis` | Complete test coverage analysis with detailed reporting |
| `quick-test` | Run all tests without coverage analysis |
| `coverage-only` | Generate coverage report only |
| `test-crate` | Test specific crate with coverage |
| `lint-all` | Run all linting and formatting checks |
| `test-integration` | Run integration tests only |
| `test-unit` | Run unit tests only |
| `test-docs` | Test documentation examples |
| `coverage-summary` | Show coverage summary |
| `install-tools` | Install required testing tools |

### Direct Script Usage

#### Comprehensive Analysis
```bash
python3 scripts/test_coverage.py
```

#### Quick Testing
```bash
# All tests and checks
./scripts/quick_test.sh

# Specific test types
./scripts/quick_test.sh unit
./scripts/quick_test.sh integration
./scripts/quick_test.sh coverage
./scripts/quick_test.sh lint
```

## Output & Reports

### Coverage Report Structure
```
coverage-report/
├── tarpaulin-report.html    # Visual HTML report
├── tarpaulin-report.json    # Machine-readable data
└── [crate-name]/           # Per-crate reports
```

### Test Coverage Report
The main report (`test_coverage_report.md`) includes:
- **Test Results Summary**: Pass/fail status for all test types
- **Coverage Summary**: Overall coverage percentage and metrics
- **Code Quality Checks**: Linting and formatting results
- **Per-Crate Results**: Individual crate test status
- **Recommendations**: Actionable items for improvement

### Example Report Output
```markdown
# Amazon Q CLI Test Coverage Report
==================================================

## Test Results Summary
- All Tests: ✅ PASSED
- Unit Tests: ✅ PASSED
- Integration Tests: ✅ PASSED
- Documentation Tests: ✅ PASSED

## Coverage Summary
- Overall Coverage: 87.5% (2450/2800 lines)

## Code Quality Checks
- Clippy: ✅ PASSED
- Format: ✅ PASSED

## Per-Crate Test Results
- chat-cli: ✅
- agent: ✅
- semantic-search-client: ✅

## Recommendations
- All tests and quality checks are passing! 🎉
```

## CI/CD Integration

### GitHub Actions
The skill includes a GitHub Actions workflow (`.github/workflows/test-coverage.yml`) that:
- Runs on every push and pull request
- Generates coverage reports
- Uploads artifacts
- Comments coverage results on PRs
- Runs daily scheduled analysis

### Coverage Tracking
- **Codecov Integration**: Automatic coverage tracking and reporting
- **Trend Analysis**: Historical coverage data
- **PR Comments**: Coverage changes in pull requests

## Configuration

### Customizing Coverage Thresholds
Edit the tarpaulin configuration in `Cargo.toml`:
```toml
[package.metadata.tarpaulin]
exclude = ["tests/*", "examples/*"]
timeout = 120
```

### Excluding Files from Coverage
Add to `.gitignore` or use tarpaulin's exclude patterns:
```bash
cargo tarpaulin --exclude-files "tests/*" --exclude-files "examples/*"
```

## Best Practices

### Writing Testable Code
1. **Small Functions**: Keep functions focused and testable
2. **Dependency Injection**: Use traits for mockable dependencies
3. **Error Handling**: Test both success and error paths
4. **Documentation**: Include examples in doc comments

### Improving Coverage
1. **Identify Gaps**: Use HTML reports to find untested code
2. **Edge Cases**: Test boundary conditions and error scenarios
3. **Integration Paths**: Ensure all user workflows are tested
4. **Regression Tests**: Add tests for bug fixes

### Performance Considerations
- Use `--skip-clean` for faster incremental coverage
- Run unit tests separately for quick feedback
- Use parallel test execution where possible

## Troubleshooting

### Common Issues

#### Tarpaulin Installation Fails
```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt-get install pkg-config libssl-dev

# Install tarpaulin
cargo install cargo-tarpaulin
```

#### Coverage Generation Timeout
```bash
# Increase timeout
cargo tarpaulin --timeout 300 --workspace
```

#### Memory Issues
```bash
# Reduce parallelism
cargo tarpaulin --workspace --jobs 1
```

### Debug Mode
Run with verbose output for debugging:
```bash
cargo tarpaulin --workspace --verbose
```

## Contributing

### Adding New Tests
1. Follow the existing test structure in `crates/*/src/tests/`
2. Use descriptive test names
3. Include both positive and negative test cases
4. Update documentation with examples

### Improving the Skill
1. Enhance the Python analysis script
2. Add new testing commands
3. Improve reporting formats
4. Extend CI/CD integration

## Resources

- [Cargo Tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Amazon Q CLI Contributing Guide](../CONTRIBUTING.md)

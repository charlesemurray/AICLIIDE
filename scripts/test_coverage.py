#!/usr/bin/env python3
"""
Comprehensive test coverage analysis for Amazon Q CLI
"""

import subprocess
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Tuple

class TestCoverageAnalyzer:
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.coverage_dir = self.project_root / "coverage-report"
        
    def run_command(self, cmd: List[str]) -> Tuple[int, str, str]:
        """Run a command and return exit code, stdout, stderr"""
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=self.project_root)
            return result.returncode, result.stdout, result.stderr
        except Exception as e:
            return 1, "", str(e)
    
    def install_coverage_tools(self) -> bool:
        """Install necessary coverage tools"""
        print("Installing coverage tools...")
        exit_code, _, stderr = self.run_command(["cargo", "install", "cargo-tarpaulin"])
        if exit_code != 0:
            print(f"Failed to install cargo-tarpaulin: {stderr}")
            return False
        return True
    
    def run_tests(self) -> Dict[str, any]:
        """Run comprehensive test suite"""
        results = {}
        
        # Run all tests
        print("Running all tests...")
        exit_code, stdout, stderr = self.run_command(["cargo", "test", "--workspace", "--all-features"])
        results["all_tests"] = {
            "exit_code": exit_code,
            "passed": exit_code == 0,
            "output": stdout,
            "errors": stderr
        }
        
        # Run unit tests
        print("Running unit tests...")
        exit_code, stdout, stderr = self.run_command(["cargo", "test", "--lib", "--workspace"])
        results["unit_tests"] = {
            "exit_code": exit_code,
            "passed": exit_code == 0,
            "output": stdout,
            "errors": stderr
        }
        
        # Run integration tests
        print("Running integration tests...")
        exit_code, stdout, stderr = self.run_command(["cargo", "test", "--test", "*", "--workspace"])
        results["integration_tests"] = {
            "exit_code": exit_code,
            "passed": exit_code == 0,
            "output": stdout,
            "errors": stderr
        }
        
        # Run doc tests
        print("Running documentation tests...")
        exit_code, stdout, stderr = self.run_command(["cargo", "test", "--doc", "--workspace"])
        results["doc_tests"] = {
            "exit_code": exit_code,
            "passed": exit_code == 0,
            "output": stdout,
            "errors": stderr
        }
        
        return results
    
    def generate_coverage_report(self) -> Dict[str, any]:
        """Generate detailed coverage report"""
        print("Generating coverage report...")
        
        # Create coverage directory
        self.coverage_dir.mkdir(exist_ok=True)
        
        # Run tarpaulin for coverage
        cmd = [
            "cargo", "tarpaulin",
            "--workspace",
            "--all-features",
            "--out", "Html",
            "--out", "Json",
            "--output-dir", str(self.coverage_dir)
        ]
        
        exit_code, stdout, stderr = self.run_command(cmd)
        
        coverage_data = {
            "exit_code": exit_code,
            "generated": exit_code == 0,
            "output": stdout,
            "errors": stderr
        }
        
        # Parse JSON coverage if available
        json_file = self.coverage_dir / "tarpaulin-report.json"
        if json_file.exists():
            try:
                with open(json_file) as f:
                    coverage_json = json.load(f)
                    coverage_data["detailed_coverage"] = coverage_json
            except Exception as e:
                coverage_data["json_parse_error"] = str(e)
        
        return coverage_data
    
    def run_lints(self) -> Dict[str, any]:
        """Run linting checks"""
        results = {}
        
        # Clippy
        print("Running clippy...")
        exit_code, stdout, stderr = self.run_command([
            "cargo", "clippy", "--workspace", "--all-features", "--", "-D", "warnings"
        ])
        results["clippy"] = {
            "exit_code": exit_code,
            "passed": exit_code == 0,
            "output": stdout,
            "errors": stderr
        }
        
        # Format check
        print("Checking formatting...")
        exit_code, stdout, stderr = self.run_command([
            "cargo", "+nightly", "fmt", "--check"
        ])
        results["format"] = {
            "exit_code": exit_code,
            "passed": exit_code == 0,
            "output": stdout,
            "errors": stderr
        }
        
        return results
    
    def analyze_crate_coverage(self) -> Dict[str, any]:
        """Analyze coverage per crate"""
        crates = []
        
        # Find all crates
        for crate_dir in (self.project_root / "crates").iterdir():
            if crate_dir.is_dir() and (crate_dir / "Cargo.toml").exists():
                crate_name = crate_dir.name
                print(f"Testing crate: {crate_name}")
                
                exit_code, stdout, stderr = self.run_command([
                    "cargo", "test", "-p", crate_name
                ])
                
                crates.append({
                    "name": crate_name,
                    "test_result": {
                        "exit_code": exit_code,
                        "passed": exit_code == 0,
                        "output": stdout,
                        "errors": stderr
                    }
                })
        
        return {"crates": crates}
    
    def generate_report(self, results: Dict[str, any]) -> str:
        """Generate a comprehensive test report"""
        report = []
        report.append("# Amazon Q CLI Test Coverage Report")
        report.append("=" * 50)
        report.append("")
        
        # Test Results Summary
        report.append("## Test Results Summary")
        test_results = results.get("test_results", {})
        for test_type, result in test_results.items():
            status = "✅ PASSED" if result.get("passed") else "❌ FAILED"
            report.append(f"- {test_type.replace('_', ' ').title()}: {status}")
        report.append("")
        
        # Coverage Summary
        coverage = results.get("coverage", {})
        if coverage.get("generated"):
            report.append("## Coverage Summary")
            detailed = coverage.get("detailed_coverage", {})
            if detailed:
                total_lines = detailed.get("files", {})
                if total_lines:
                    covered = sum(f.get("covered", 0) for f in total_lines.values())
                    total = sum(f.get("coverable", 0) for f in total_lines.values())
                    percentage = (covered / total * 100) if total > 0 else 0
                    report.append(f"- Overall Coverage: {percentage:.2f}% ({covered}/{total} lines)")
            report.append("")
        
        # Lint Results
        lint_results = results.get("lint_results", {})
        report.append("## Code Quality Checks")
        for lint_type, result in lint_results.items():
            status = "✅ PASSED" if result.get("passed") else "❌ FAILED"
            report.append(f"- {lint_type.title()}: {status}")
        report.append("")
        
        # Per-Crate Results
        crate_analysis = results.get("crate_analysis", {})
        if crate_analysis.get("crates"):
            report.append("## Per-Crate Test Results")
            for crate in crate_analysis["crates"]:
                status = "✅" if crate["test_result"]["passed"] else "❌"
                report.append(f"- {crate['name']}: {status}")
            report.append("")
        
        # Recommendations
        report.append("## Recommendations")
        failed_tests = [k for k, v in test_results.items() if not v.get("passed")]
        if failed_tests:
            report.append("### Failed Tests")
            for test in failed_tests:
                report.append(f"- Fix {test.replace('_', ' ')}")
        
        failed_lints = [k for k, v in lint_results.items() if not v.get("passed")]
        if failed_lints:
            report.append("### Code Quality Issues")
            for lint in failed_lints:
                report.append(f"- Address {lint} issues")
        
        if not failed_tests and not failed_lints:
            report.append("- All tests and quality checks are passing! 🎉")
        
        return "\n".join(report)
    
    def run_full_analysis(self) -> None:
        """Run complete test coverage analysis"""
        print("Starting comprehensive test coverage analysis...")
        
        # Install tools
        if not self.install_coverage_tools():
            print("Failed to install coverage tools")
            sys.exit(1)
        
        results = {}
        
        # Run tests
        results["test_results"] = self.run_tests()
        
        # Generate coverage
        results["coverage"] = self.generate_coverage_report()
        
        # Run lints
        results["lint_results"] = self.run_lints()
        
        # Analyze per crate
        results["crate_analysis"] = self.analyze_crate_coverage()
        
        # Generate report
        report = self.generate_report(results)
        
        # Save report
        report_file = self.project_root / "test_coverage_report.md"
        with open(report_file, "w") as f:
            f.write(report)
        
        print(f"\nTest coverage analysis complete!")
        print(f"Report saved to: {report_file}")
        print(f"Coverage HTML report: {self.coverage_dir / 'tarpaulin-report.html'}")
        
        # Print summary
        print("\n" + "=" * 50)
        print(report)

if __name__ == "__main__":
    project_root = os.getcwd()
    analyzer = TestCoverageAnalyzer(project_root)
    analyzer.run_full_analysis()

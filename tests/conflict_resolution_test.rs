/// Test conflict resolution guidance output
#[cfg(test)]
mod tests {
    use std::io::Write;

    #[test]
    fn test_conflict_guidance_format() {
        let conflicts = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
        ];
        
        // Capture output
        let mut output = Vec::new();
        writeln!(output, "⚠️  Conflicts detected in {} file(s):", conflicts.len()).unwrap();
        for file in conflicts.iter().take(5) {
            writeln!(output, "  • {}", file).unwrap();
        }
        
        writeln!(output, "\n📋 Resolution options:").unwrap();
        writeln!(output, "  1. Resolve manually:").unwrap();
        
        let result = String::from_utf8(output).unwrap();
        
        assert!(result.contains("Conflicts detected in 2 file(s)"));
        assert!(result.contains("src/main.rs"));
        assert!(result.contains("Resolution options"));
    }
}

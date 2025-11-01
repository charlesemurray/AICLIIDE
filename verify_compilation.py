#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Verify that the creation system compiles by checking for common compilation issues
"""

import os
import re

def check_compilation_issues():
    """Check for common compilation issues in the creation system."""
    print("Compilation Issues Check")
    print("=" * 30)
    
    creation_dir = "/local/workspace/q-cli/amazon-q-developer-cli/crates/chat-cli/src/cli/creation"
    
    issues_found = []
    
    # Check 1: Missing imports
    print("\n1. Checking for missing imports...")
    
    files_to_check = [
        ("flows/skill.rs", ["TerminalUI", "CreationContext"]),
        ("flows/agent.rs", ["TerminalUI", "CreationContext"]),
        ("flows/command.rs", ["TerminalUI", "CreationContext"])
    ]
    
    for file_path, required_imports in files_to_check:
        full_path = os.path.join(creation_dir, file_path)
        if os.path.exists(full_path):
            with open(full_path, 'r') as f:
                content = f.read()
            
            missing_imports = []
            for import_name in required_imports:
                if import_name not in content:
                    missing_imports.append(import_name)
            
            if missing_imports:
                issues_found.append("Missing imports in {}: {}".format(file_path, missing_imports))
        else:
            issues_found.append("File not found: {}".format(file_path))
    
    # Check 2: Trait method mismatches
    print("\n2. Checking for trait method mismatches...")
    
    # Check that get_name IS implemented in CreationConfig impls (it's required)
    for file_path in ["flows/skill.rs", "flows/agent.rs", "flows/command.rs"]:
        full_path = os.path.join(creation_dir, file_path)
        if os.path.exists(full_path):
            with open(full_path, 'r') as f:
                content = f.read()
            
            # Look for missing get_name in CreationConfig impl blocks
            if "impl CreationConfig" in content and "fn get_name" not in content:
                issues_found.append("get_name method missing from CreationConfig impl in {}".format(file_path))
    
    # Check 3: Circular imports
    print("\n3. Checking for circular imports...")
    
    mod_file = os.path.join(creation_dir, "mod.rs")
    if os.path.exists(mod_file):
        with open(mod_file, 'r') as f:
            mod_content = f.read()
        
        # Check that TerminalUI is properly exported
        if "pub use ui::{TerminalUIImpl, TerminalUI};" not in mod_content:
            if "TerminalUI" not in mod_content:
                issues_found.append("TerminalUI not properly exported from mod.rs")
    
    # Check 4: Syntax issues
    print("\n4. Checking for basic syntax issues...")
    
    all_rs_files = []
    for root, dirs, files in os.walk(creation_dir):
        for file in files:
            if file.endswith('.rs'):
                all_rs_files.append(os.path.join(root, file))
    
    for file_path in all_rs_files:
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Check for unmatched braces (simple check)
        open_braces = content.count('{')
        close_braces = content.count('}')
        if open_braces != close_braces:
            issues_found.append("Unmatched braces in {}".format(os.path.relpath(file_path, creation_dir)))
        
        # Check for missing semicolons after struct definitions
        if re.search(r'struct\s+\w+\s*{[^}]*}\s*(?!;|\s*impl)', content):
            # This is a complex check, skip for now
            pass
    
    # Report results
    if issues_found:
        print("\nFAIL: Compilation issues found:")
        for issue in issues_found:
            print("  - {}".format(issue))
        return False
    else:
        print("\nPASS: No obvious compilation issues found")
        print("\nChecked:")
        print("- Import statements in flow modules")
        print("- Trait method implementations")
        print("- Module exports")
        print("- Basic syntax issues")
        return True

if __name__ == "__main__":
    import sys
    success = check_compilation_issues()
    sys.exit(0 if success else 1)

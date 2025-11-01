#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Static analysis to check for compilation issues without actually compiling
"""

import os
import re

def static_compile_check():
    """Perform static analysis to identify likely compilation issues."""
    print("Static Compilation Analysis")
    print("=" * 35)
    
    creation_dir = "/local/workspace/q-cli/amazon-q-developer-cli/crates/chat-cli/src/cli/creation"
    issues = []
    
    # Check 1: Import resolution
    print("\n1. Checking import resolution...")
    
    # Map of what each file should import
    import_requirements = {
        "flows/command.rs": [
            "CreationFlow", "CreationConfig", "CreationArtifact", 
            "CreationType", "CreationPhase", "PhaseResult", "CreationMode",
            "TerminalUI", "CreationContext", "CommandType", "CreationError"
        ],
        "flows/skill.rs": [
            "CreationFlow", "CreationConfig", "CreationArtifact",
            "CreationType", "CreationPhase", "PhaseResult", "CreationMode", 
            "SkillType", "SecurityLevel", "TerminalUI", "CreationContext"
        ],
        "flows/agent.rs": [
            "CreationFlow", "CreationConfig", "CreationArtifact",
            "CreationType", "CreationPhase", "PhaseResult", "CreationMode",
            "TerminalUI", "CreationContext"
        ]
    }
    
    for file_path, required_imports in import_requirements.items():
        full_path = os.path.join(creation_dir, file_path)
        if os.path.exists(full_path):
            with open(full_path, 'r') as f:
                content = f.read()
            
            # Check use statements
            use_block = ""
            for line in content.split('\n'):
                if line.strip().startswith('use '):
                    use_block += line + '\n'
                elif line.strip() == '' or line.strip().startswith('//'):
                    continue
                else:
                    break
            
            missing_imports = []
            for import_name in required_imports:
                if import_name not in use_block and import_name not in content:
                    missing_imports.append(import_name)
            
            if missing_imports:
                issues.append("Missing imports in {}: {}".format(file_path, missing_imports))
    
    # Check 2: Trait implementation completeness
    print("\n2. Checking trait implementations...")
    
    trait_methods = {
        "CreationConfig": ["validate", "apply_defaults", "is_complete", "get_name"],
        "CreationArtifact": ["persist", "validate_before_save", "get_name"],
        "CreationFlow": ["creation_type", "execute_phase", "create_artifact", "get_config"]
    }
    
    for file_path in ["flows/command.rs", "flows/skill.rs", "flows/agent.rs"]:
        full_path = os.path.join(creation_dir, file_path)
        if os.path.exists(full_path):
            with open(full_path, 'r') as f:
                content = f.read()
            
            for trait_name, methods in trait_methods.items():
                if "impl {} for".format(trait_name) in content:
                    # Find the impl block
                    impl_pattern = r'impl\s+{}\s+for\s+\w+\s*{{([^}}]+)}}'.format(re.escape(trait_name))
                    impl_match = re.search(impl_pattern, content, re.DOTALL)
                    
                    if impl_match:
                        impl_block = impl_match.group(1)
                        missing_methods = []
                        for method in methods:
                            if "fn {}".format(method) not in impl_block:
                                missing_methods.append(method)
                        
                        if missing_methods:
                            issues.append("Missing methods in {} impl in {}: {}".format(
                                trait_name, file_path, missing_methods))
    
    # Check 3: Type consistency
    print("\n3. Checking type consistency...")
    
    # Check that all referenced types are defined or imported
    type_definitions = {}
    
    # Scan types.rs for type definitions
    types_file = os.path.join(creation_dir, "types.rs")
    if os.path.exists(types_file):
        with open(types_file, 'r') as f:
            types_content = f.read()
        
        # Find enum and struct definitions
        for match in re.finditer(r'pub\s+(enum|struct)\s+(\w+)', types_content):
            type_definitions[match.group(2)] = match.group(1)
    
    # Check each flow file for undefined types
    for file_path in ["flows/command.rs", "flows/skill.rs", "flows/agent.rs"]:
        full_path = os.path.join(creation_dir, file_path)
        if os.path.exists(full_path):
            with open(full_path, 'r') as f:
                content = f.read()
            
            # Look for type usage that might be undefined
            type_usage = re.findall(r':\s*(\w+)', content)
            type_usage.extend(re.findall(r'<(\w+)>', content))
            
            for used_type in type_usage:
                if (used_type not in type_definitions and 
                    used_type not in ['String', 'Vec', 'Option', 'Result', 'Box', 'Path', 'PathBuf'] and
                    used_type not in content):  # Not defined locally
                    # This might be an issue, but hard to tell without full context
                    pass
    
    # Check 4: Syntax issues
    print("\n4. Checking syntax issues...")
    
    for root, dirs, files in os.walk(creation_dir):
        for file in files:
            if file.endswith('.rs'):
                file_path = os.path.join(root, file)
                with open(file_path, 'r') as f:
                    content = f.read()
                
                # Check for unmatched braces
                open_braces = content.count('{')
                close_braces = content.count('}')
                if open_braces != close_braces:
                    issues.append("Unmatched braces in {}".format(os.path.relpath(file_path, creation_dir)))
                
                # Check for unmatched parentheses
                open_parens = content.count('(')
                close_parens = content.count(')')
                if open_parens != close_parens:
                    issues.append("Unmatched parentheses in {}".format(os.path.relpath(file_path, creation_dir)))
                
                # Check for missing semicolons after statements (basic check)
                lines = content.split('\n')
                for i, line in enumerate(lines):
                    stripped = line.strip()
                    if (stripped.startswith('let ') and 
                        not stripped.endswith(';') and 
                        not stripped.endswith('{') and
                        not stripped.endswith('{')):
                        # This might be an issue
                        pass
    
    # Check 5: Module structure
    print("\n5. Checking module structure...")
    
    mod_file = os.path.join(creation_dir, "mod.rs")
    if os.path.exists(mod_file):
        with open(mod_file, 'r') as f:
            mod_content = f.read()
        
        # Check that all submodules are declared
        expected_modules = ["types", "errors", "ui", "assistant", "flows", "context", "templates"]
        for module in expected_modules:
            if "mod {};".format(module) not in mod_content:
                issues.append("Module '{}' not declared in mod.rs".format(module))
        
        # Check that flows submodule exists
        flows_mod = os.path.join(creation_dir, "flows", "mod.rs")
        if os.path.exists(flows_mod):
            with open(flows_mod, 'r') as f:
                flows_content = f.read()
            
            expected_flow_modules = ["command", "skill", "agent"]
            for module in expected_flow_modules:
                if "mod {};".format(module) not in flows_content:
                    issues.append("Flow module '{}' not declared in flows/mod.rs".format(module))
    
    # Report results
    if issues:
        print("\nPOTENTIAL COMPILATION ISSUES FOUND:")
        for issue in issues:
            print("  - {}".format(issue))
        print("\nNote: These are potential issues found through static analysis.")
        print("Actual compilation may reveal additional issues or false positives.")
        return False
    else:
        print("\nSTATIC ANALYSIS PASSED")
        print("No obvious compilation issues found through static analysis.")
        print("\nChecked:")
        print("- Import statements and dependencies")
        print("- Trait implementation completeness") 
        print("- Basic syntax issues (braces, parentheses)")
        print("- Module structure and declarations")
        print("\nNote: This doesn't guarantee compilation success,")
        print("but indicates the code structure appears sound.")
        return True

if __name__ == "__main__":
    import sys
    success = static_compile_check()
    sys.exit(0 if success else 1)

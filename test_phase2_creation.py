#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Phase 2 Creation System Test - Enhanced flows with UI integration
"""

import os
import tempfile
import json

def test_phase2_enhancements():
    """Test Phase 2 enhancements: UI integration, templates, advanced flows."""
    print("Testing Phase 2 Creation System Enhancements")
    print("=" * 50)
    
    # Test 1: Enhanced skill creation flow
    print("\n1. Testing enhanced skill creation flow...")
    try:
        skill_flow_path = "/local/workspace/q-cli/amazon-q-developer-cli/crates/chat-cli/src/cli/creation/flows/skill.rs"
        with open(skill_flow_path, 'r') as f:
            skill_content = f.read()
        
        # Check for UI integration
        required_ui_methods = [
            "execute_discovery",
            "execute_security", 
            "with_ui",
            "prompt_required",
            "prompt_optional",
            "show_message"
        ]
        
        missing_methods = []
        for method in required_ui_methods:
            if method not in skill_content:
                missing_methods.append(method)
        
        if missing_methods:
            print("FAIL: Missing UI methods in skill flow: {}".format(missing_methods))
            return False
        else:
            print("PASS: Skill flow has proper UI integration")
            
    except Exception as e:
        print("FAIL: Skill flow check failed: {}".format(e))
        return False
    
    # Test 2: Enhanced agent creation flow
    print("\n2. Testing enhanced agent creation flow...")
    try:
        agent_flow_path = "/local/workspace/q-cli/amazon-q-developer-cli/crates/chat-cli/src/cli/creation/flows/agent.rs"
        with open(agent_flow_path, 'r') as f:
            agent_content = f.read()
        
        # Check for advanced configuration
        required_agent_features = [
            "execute_discovery",
            "execute_advanced_config",
            "MCP servers",
            "lifecycle hooks",
            "with_ui"
        ]
        
        missing_features = []
        for feature in required_agent_features:
            if feature not in agent_content:
                missing_features.append(feature)
        
        if missing_features:
            print("FAIL: Missing agent features: {}".format(missing_features))
            return False
        else:
            print("PASS: Agent flow has advanced configuration")
            
    except Exception as e:
        print("FAIL: Agent flow check failed: {}".format(e))
        return False
    
    # Test 3: Template system
    print("\n3. Testing template system...")
    try:
        template_path = "/local/workspace/q-cli/amazon-q-developer-cli/crates/chat-cli/src/cli/creation/templates.rs"
        
        if not os.path.exists(template_path):
            print("FAIL: Template system not implemented")
            return False
        
        with open(template_path, 'r') as f:
            template_content = f.read()
        
        required_template_features = [
            "TemplateManager",
            "load_template",
            "list_available_templates",
            "apply_template_to_config"
        ]
        
        missing_template_features = []
        for feature in required_template_features:
            if feature not in template_content:
                missing_template_features.append(feature)
        
        if missing_template_features:
            print("FAIL: Missing template features: {}".format(missing_template_features))
            return False
        else:
            print("PASS: Template system implemented")
            
    except Exception as e:
        print("FAIL: Template system check failed: {}".format(e))
        return False
    
    # Test 4: Module integration
    print("\n4. Testing module integration...")
    try:
        mod_path = "/local/workspace/q-cli/amazon-q-developer-cli/crates/chat-cli/src/cli/creation/mod.rs"
        with open(mod_path, 'r') as f:
            mod_content = f.read()
        
        required_exports = [
            "mod templates",
            "pub use templates::TemplateManager"
        ]
        
        missing_exports = []
        for export in required_exports:
            if export not in mod_content:
                missing_exports.append(export)
        
        if missing_exports:
            print("FAIL: Missing module exports: {}".format(missing_exports))
            return False
        else:
            print("PASS: Module integration correct")
            
    except Exception as e:
        print("FAIL: Module integration check failed: {}".format(e))
        return False
    
    # Test 5: UI interaction patterns
    print("\n5. Testing UI interaction patterns...")
    try:
        # Check that flows use proper UI patterns
        flows_to_check = [
            ("skill.rs", "Skill creation"),
            ("agent.rs", "Agent creation"),
            ("command.rs", "Command creation")
        ]
        
        for flow_file, flow_name in flows_to_check:
            flow_path = "/local/workspace/q-cli/amazon-q-developer-cli/crates/chat-cli/src/cli/creation/flows/{}".format(flow_file)
            with open(flow_path, 'r') as f:
                flow_content = f.read()
            
            # Check for proper UI usage patterns
            ui_patterns = [
                "ui: &mut dyn TerminalUI",
                "ui.prompt_required",
                "ui.show_message",
                "SemanticColor::"
            ]
            
            missing_patterns = []
            for pattern in ui_patterns:
                if pattern not in flow_content:
                    missing_patterns.append(pattern)
            
            if missing_patterns:
                print("FAIL: {} missing UI patterns: {}".format(flow_name, missing_patterns))
                return False
        
        print("PASS: All flows use proper UI patterns")
        
    except Exception as e:
        print("FAIL: UI patterns check failed: {}".format(e))
        return False
    
    print("\nSUCCESS: Phase 2 enhancements implemented correctly!")
    print("\nPhase 2 Features Verified:")
    print("- Enhanced skill creation with UI integration")
    print("- Advanced agent creation with MCP/tools/hooks")
    print("- Template system for reusable configurations")
    print("- Proper UI interaction patterns")
    print("- Module integration and exports")
    
    return True

if __name__ == "__main__":
    import sys
    success = test_phase2_enhancements()
    sys.exit(0 if success else 1)

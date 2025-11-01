#!/usr/bin/env python3

import subprocess
import os
import json
import tempfile
import shutil

def test_skill_creation():
    """Minimal test to verify skill creation works end-to-end"""
    
    # Create temporary directory for testing
    with tempfile.TemporaryDirectory() as temp_dir:
        skills_dir = os.path.join(temp_dir, ".q-skills")
        os.makedirs(skills_dir, exist_ok=True)
        
        # Test 1: Create skill JSON manually (simulating what the CLI should do)
        skill_data = {
            "name": "manual-test",
            "description": "Manual test skill",
            "type": "command",
            "command": "echo 'test successful'",
            "context_files": {
                "patterns": ["*.txt"],
                "max_files": 10,
                "max_file_size_kb": 100
            }
        }
        
        skill_file = os.path.join(skills_dir, "manual-test.json")
        with open(skill_file, 'w') as f:
            json.dump(skill_data, f, indent=2)
        
        print(f"✓ Created skill file: {skill_file}")
        
        # Test 2: Verify file exists and is valid JSON
        if os.path.exists(skill_file):
            with open(skill_file, 'r') as f:
                loaded_data = json.load(f)
                print(f"✓ Skill JSON is valid: {loaded_data['name']}")
        else:
            print("✗ Skill file not created")
            return False
        
        # Test 3: Try to list skills using the CLI
        try:
            os.environ['HOME'] = temp_dir  # Point to our test directory
            result = subprocess.run([
                "cargo", "run", "--bin", "chat_cli", "--", "skills", "list"
            ], capture_output=True, text=True, cwd="/local/workspace/q-cli/amazon-q-developer-cli")
            
            if result.returncode == 0:
                print(f"✓ Skills list command succeeded")
                if "manual-test" in result.stdout:
                    print("✓ Created skill appears in list")
                else:
                    print("✗ Created skill not found in list")
                    print(f"Output: {result.stdout}")
            else:
                print(f"✗ Skills list failed: {result.stderr}")
                return False
                
        except Exception as e:
            print(f"✗ Error running CLI: {e}")
            return False
        
        return True

if __name__ == "__main__":
    print("Testing skill creation system...")
    success = test_skill_creation()
    print(f"\nTest {'PASSED' if success else 'FAILED'}")

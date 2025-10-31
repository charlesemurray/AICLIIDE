#!/usr/bin/env python3
"""
Error demo skill implementation
Demonstrates colored error output using ANSI escape codes
"""
import sys
import json

# ANSI color codes
class Colors:
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    WHITE = '\033[97m'
    GRAY = '\033[90m'
    BOLD = '\033[1m'
    DIM = '\033[2m'
    RESET = '\033[0m'

def format_error(error_type, message, suggestions=None, context=None):
    """Format an error with colors and suggestions"""
    output = []
    
    # Error header with type indicator
    type_indicators = {
        'auth': 'AUTH',
        'network': 'NETWORK',
        'file': 'FILE', 
        'input': 'INPUT',
        'system': 'SYSTEM',
        'tool': 'TOOL'
    }
    
    indicator = type_indicators.get(error_type, 'ERROR')
    output.append(f"{Colors.RED}✗ {indicator}: {message}{Colors.RESET}")
    
    # Add context if provided
    if context:
        output.append(f"{Colors.GRAY}Context: {context}{Colors.RESET}")
    
    # Add suggestions if any
    if suggestions:
        output.append("")
        output.append(f"{Colors.BLUE}Suggestions:{Colors.RESET}")
        for suggestion in suggestions:
            output.append(f"  {Colors.GRAY}•{Colors.RESET} {Colors.WHITE}{suggestion}{Colors.RESET}")
    
    return "\n".join(output)

def get_error_examples():
    """Get all error type examples"""
    return {
        'auth': {
            'message': 'Authentication token has expired',
            'suggestions': [
                "Run 'q login' to authenticate",
                "Check your internet connection", 
                "Verify your AWS credentials"
            ]
        },
        'network': {
            'message': 'Failed to connect to API server',
            'context': 'Endpoint: https://api.example.com',
            'suggestions': [
                "Check your internet connection",
                "Try again in a few moments",
                "Verify the service is available"
            ]
        },
        'file': {
            'message': 'Permission denied',
            'context': 'Path: /etc/secure/config.json',
            'suggestions': [
                "Check if the file exists",
                "Verify file permissions",
                "Ensure the directory is accessible"
            ]
        },
        'input': {
            'message': 'Invalid command syntax: missing required argument',
            'context': 'Command: /example --missing-arg',
            'suggestions': [
                "Check the command syntax",
                "Use --help for usage information",
                "Verify all required parameters are provided"
            ]
        },
        'system': {
            'message': 'Internal system error occurred',
            'context': 'Component: session_manager',
            'suggestions': [
                "Restart the application",
                "Check system logs",
                "Contact support if the issue persists"
            ]
        },
        'tool': {
            'message': 'Tool execution timed out after 30 seconds',
            'context': 'Tool: git',
            'suggestions': [
                "Check tool permissions",
                "Verify tool dependencies are installed", 
                "Try running the tool manually"
            ]
        }
    }

def main():
    """Main execution function"""
    try:
        # Parse parameters from command line
        params = {}
        if len(sys.argv) > 1:
            params = json.loads(sys.argv[1])
        
        error_type = params.get('error_type')
        examples = get_error_examples()
        
        if error_type and error_type in examples:
            # Show specific error type
            example = examples[error_type]
            print(format_error(
                error_type,
                example['message'],
                example.get('suggestions'),
                example.get('context')
            ))
        else:
            # Show all error types
            print(f"{Colors.BOLD}{Colors.MAGENTA}Error Display Demo{Colors.RESET}")
            print()
            
            for err_type, example in examples.items():
                print(f"{Colors.BOLD}{Colors.MAGENTA}{err_type.upper()}:{Colors.RESET}")
                print(format_error(
                    err_type,
                    example['message'],
                    example.get('suggestions'),
                    example.get('context')
                ))
                print()
            
            print(f"{Colors.BLUE}Use error_type parameter to see specific examples{Colors.RESET}")
            print(f"{Colors.GRAY}Example: @error-demo auth{Colors.RESET}")
    
    except Exception as e:
        print(f"{Colors.RED}Error in error-demo skill: {e}{Colors.RESET}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()

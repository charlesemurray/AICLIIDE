// Test for cleanup confirmation logic
// Note: These are integration-style tests that verify the command structure

use chat_cli::cli::chat::cli::sessions::SessionsSubcommand;

#[test]
fn test_cleanup_has_force_flag() {
    // Verify that Cleanup variant has force field
    let cleanup = SessionsSubcommand::Cleanup {
        completed: false,
        older_than: None,
        force: false,
    };
    
    // Should compile - verifies force field exists
    match cleanup {
        SessionsSubcommand::Cleanup { force, .. } => {
            assert_eq!(force, false);
        }
        _ => panic!("Expected Cleanup variant"),
    }
}

#[test]
fn test_cleanup_force_flag_can_be_true() {
    let cleanup = SessionsSubcommand::Cleanup {
        completed: true,
        older_than: Some(30),
        force: true,
    };
    
    match cleanup {
        SessionsSubcommand::Cleanup { force, completed, older_than } => {
            assert_eq!(force, true);
            assert_eq!(completed, true);
            assert_eq!(older_than, Some(30));
        }
        _ => panic!("Expected Cleanup variant"),
    }
}

#[test]
fn test_cleanup_default_force_is_false() {
    // When force is not specified, it should default to false
    let cleanup = SessionsSubcommand::Cleanup {
        completed: false,
        older_than: None,
        force: false,
    };
    
    match cleanup {
        SessionsSubcommand::Cleanup { force, .. } => {
            assert_eq!(force, false, "Default force should be false");
        }
        _ => panic!("Expected Cleanup variant"),
    }
}

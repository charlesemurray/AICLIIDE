#[cfg(test)]
mod test_enhanced_transitions {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_transition_manager_creation() {
        let manager = TransitionManager::new();
        assert_eq!(manager.get_transition_history(10).len(), 0);
    }

    #[test]
    fn test_transition_with_confirmation() {
        let mut manager = TransitionManager::new();
        let result = manager.transition_with_confirmation(
            ConversationMode::Interactive,
            ConversationMode::ExecutePlan,
            ConversationModeTrigger::UserCommand
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_transition_preview() {
        let manager = TransitionManager::new();
        let preview = manager.show_transition_preview(
            ConversationMode::Interactive,
            ConversationMode::Review
        );
        assert!(preview.contains("Interactive → Review"));
        assert!(preview.contains("analyze and provide feedback"));
    }

    #[test]
    fn test_transition_history_tracking() {
        let mut manager = TransitionManager::new();
        manager.add_transition(ModeTransition {
            from: ConversationMode::Interactive,
            to: ConversationMode::ExecutePlan,
            trigger: ConversationModeTrigger::Auto,
            timestamp: SystemTime::now(),
            user_confirmed: false,
        });

        let history = manager.get_transition_history(5);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].from, ConversationMode::Interactive);
        assert_eq!(history[0].to, ConversationMode::ExecutePlan);
    }

    #[test]
    fn test_undo_last_transition() {
        let mut manager = TransitionManager::new();
        manager.add_transition(ModeTransition {
            from: ConversationMode::Interactive,
            to: ConversationMode::ExecutePlan,
            trigger: ConversationModeTrigger::Auto,
            timestamp: SystemTime::now(),
            user_confirmed: false,
        });

        let result = manager.undo_last_transition();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConversationMode::Interactive);
    }

    #[test]
    fn test_undo_user_confirmed_transition_fails() {
        let mut manager = TransitionManager::new();
        manager.add_transition(ModeTransition {
            from: ConversationMode::Interactive,
            to: ConversationMode::ExecutePlan,
            trigger: ConversationModeTrigger::UserCommand,
            timestamp: SystemTime::now(),
            user_confirmed: true,
        });

        let result = manager.undo_last_transition();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot undo user-confirmed"));
    }

    #[test]
    fn test_destructive_transition_requires_confirmation() {
        let manager = TransitionManager::new();
        let requires_confirmation = manager.requires_confirmation(
            ConversationMode::ExecutePlan,
            ConversationMode::Review
        );
        assert_eq!(requires_confirmation, true);
    }

    #[test]
    fn test_safe_transition_no_confirmation() {
        let manager = TransitionManager::new();
        let requires_confirmation = manager.requires_confirmation(
            ConversationMode::Interactive,
            ConversationMode::ExecutePlan
        );
        assert_eq!(requires_confirmation, false);
    }

    #[test]
    fn test_transition_animation_indicator() {
        let manager = TransitionManager::new();
        let indicator = manager.get_transition_indicator(
            ConversationMode::Interactive,
            ConversationMode::ExecutePlan
        );
        assert!(indicator.contains("💬 → 🚀"));
    }
}

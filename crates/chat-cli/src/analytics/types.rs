use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationAnalyticsEvent {
    pub timestamp: u64,
    pub session_id: String,
    pub event: AnalyticsEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum AnalyticsEventType {
    ContinuationPrompt {
        step_number: u32,
        task_context: String,
        total_steps_estimated: Option<u32>,
    },
    UserResponse {
        prompt_type: PromptType,
        response: UserResponseType,
        response_time_ms: u64,
    },
    QuestionAsked {
        question_type: QuestionType,
        by_llm: bool,
        conversation_position: ConversationPosition,
    },
    SessionFlow {
        event_type: SessionEventType,
        at_message_count: u32,
        duration_ms: Option<u64>,
    },
    ModeTransition {
        from_mode: Option<ConversationMode>,
        to_mode: ConversationMode,
        trigger: ModeTransitionTrigger,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromptType {
    ContinuationPrompt,
    ToolApproval,
    QuestionResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserResponseType {
    Approved,
    Rejected,
    Modified,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestionType {
    Clarification,
    Permission,
    Technical,
    Requirements,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversationPosition {
    Early,
    Mid,
    Late,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionEventType {
    Started,
    Paused,
    Resumed,
    Abandoned,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversationMode {
    Planning,
    Implementation,
    Review,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModeTransitionTrigger {
    Auto,
    UserCommand,
    LLMDecision,
}

impl ConversationAnalyticsEvent {
    pub fn new(session_id: String, event: AnalyticsEventType) -> Self {
        Self {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            session_id,
            event,
        }
    }

    pub fn continuation_prompt(
        session_id: String,
        step_number: u32,
        task_context: String,
        total_steps_estimated: Option<u32>,
    ) -> Self {
        Self::new(
            session_id,
            AnalyticsEventType::ContinuationPrompt {
                step_number,
                task_context,
                total_steps_estimated,
            },
        )
    }

    pub fn user_response(
        session_id: String,
        prompt_type: PromptType,
        response: UserResponseType,
        response_time_ms: u64,
    ) -> Self {
        Self::new(
            session_id,
            AnalyticsEventType::UserResponse {
                prompt_type,
                response,
                response_time_ms,
            },
        )
    }

    pub fn question_asked(
        session_id: String,
        question_type: QuestionType,
        by_llm: bool,
        conversation_position: ConversationPosition,
    ) -> Self {
        Self::new(
            session_id,
            AnalyticsEventType::QuestionAsked {
                question_type,
                by_llm,
                conversation_position,
            },
        )
    }

    pub fn session_flow(
        session_id: String,
        event_type: SessionEventType,
        at_message_count: u32,
        duration_ms: Option<u64>,
    ) -> Self {
        Self::new(
            session_id,
            AnalyticsEventType::SessionFlow {
                event_type,
                at_message_count,
                duration_ms,
            },
        )
    }

    pub fn mode_transition(
        session_id: String,
        from_mode: Option<ConversationMode>,
        to_mode: ConversationMode,
        trigger: ModeTransitionTrigger,
    ) -> Self {
        Self::new(
            session_id,
            AnalyticsEventType::ModeTransition {
                from_mode,
                to_mode,
                trigger,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

    use super::*;

    #[test]
    fn test_continuation_prompt_serialization() {
        let event = ConversationAnalyticsEvent::continuation_prompt(
            "session_123".to_string(),
            5,
            "authentication system".to_string(),
            Some(10),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ConversationAnalyticsEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
        assert!(json.contains("ContinuationPrompt"));
        assert!(json.contains("session_123"));
        assert!(json.contains("authentication system"));
    }

    #[test]
    fn test_user_response_serialization() {
        let event = ConversationAnalyticsEvent::user_response(
            "session_456".to_string(),
            PromptType::ToolApproval,
            UserResponseType::Approved,
            2500,
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ConversationAnalyticsEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
        assert!(json.contains("UserResponse"));
        assert!(json.contains("ToolApproval"));
        assert!(json.contains("Approved"));
    }

    #[test]
    fn test_question_asked_serialization() {
        let event = ConversationAnalyticsEvent::question_asked(
            "session_789".to_string(),
            QuestionType::Clarification,
            true,
            ConversationPosition::Early,
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ConversationAnalyticsEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
        assert!(json.contains("QuestionAsked"));
        assert!(json.contains("Clarification"));
        assert!(json.contains("Early"));
    }

    #[test]
    fn test_session_flow_serialization() {
        let event =
            ConversationAnalyticsEvent::session_flow("session_abc".to_string(), SessionEventType::Started, 0, None);

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ConversationAnalyticsEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
        assert!(json.contains("SessionFlow"));
        assert!(json.contains("Started"));
    }

    #[test]
    fn test_mode_transition_serialization() {
        let event = ConversationAnalyticsEvent::mode_transition(
            "session_def".to_string(),
            Some(ConversationMode::Planning),
            ConversationMode::Implementation,
            ModeTransitionTrigger::UserCommand,
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ConversationAnalyticsEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
        assert!(json.contains("ModeTransition"));
        assert!(json.contains("Planning"));
        assert!(json.contains("Implementation"));
        assert!(json.contains("UserCommand"));
    }

    #[test]
    fn test_all_enum_variants_serialize() {
        // Test all PromptType variants
        let prompt_types = vec![
            PromptType::ContinuationPrompt,
            PromptType::ToolApproval,
            PromptType::QuestionResponse,
        ];
        for pt in prompt_types {
            let json = serde_json::to_string(&pt).unwrap();
            let _: PromptType = serde_json::from_str(&json).unwrap();
        }

        // Test all UserResponseType variants
        let response_types = vec![
            UserResponseType::Approved,
            UserResponseType::Rejected,
            UserResponseType::Modified,
            UserResponseType::Abandoned,
        ];
        for rt in response_types {
            let json = serde_json::to_string(&rt).unwrap();
            let _: UserResponseType = serde_json::from_str(&json).unwrap();
        }

        // Test all QuestionType variants
        let question_types = vec![
            QuestionType::Clarification,
            QuestionType::Permission,
            QuestionType::Technical,
            QuestionType::Requirements,
        ];
        for qt in question_types {
            let json = serde_json::to_string(&qt).unwrap();
            let _: QuestionType = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_timestamp_generation() {
        let event1 = ConversationAnalyticsEvent::new(
            "test".to_string(),
            AnalyticsEventType::SessionFlow {
                event_type: SessionEventType::Started,
                at_message_count: 0,
                duration_ms: None,
            },
        );

        std::thread::sleep(std::time::Duration::from_millis(1));

        let event2 = ConversationAnalyticsEvent::new(
            "test".to_string(),
            AnalyticsEventType::SessionFlow {
                event_type: SessionEventType::Started,
                at_message_count: 0,
                duration_ms: None,
            },
        );

        assert!(event2.timestamp > event1.timestamp);
    }
}

//! Tower-based LLM service for rate-limited, prioritized API calls

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;
use crate::api_client::ApiClient;
use crate::api_client::model::ConversationState;
use crate::cli::chat::parser::{SendMessageStream, SendMessageError};

/// Request to LLM service
#[derive(Clone)]
pub struct LLMRequest {
    pub conversation_state: ConversationState,
    pub priority: RequestPriority,
}

/// Priority level for LLM requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    High = 0,  // Active session
    Low = 1,   // Background session
}

/// Tower service for making LLM API calls
#[derive(Clone)]
pub struct LLMService {
    client: ApiClient,
}

impl LLMService {
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }
}

impl Service<LLMRequest> for LLMService {
    type Response = SendMessageStream;
    type Error = SendMessageError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Always ready - actual rate limiting handled by Tower layers
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: LLMRequest) -> Self::Future {
        let client = self.client.clone();
        
        Box::pin(async move {
            let request_metadata_lock = std::sync::Arc::new(tokio::sync::Mutex::new(None));
            
            SendMessageStream::send_message(
                &client,
                req.conversation_state,
                request_metadata_lock,
                None
            ).await
        })
    }
}

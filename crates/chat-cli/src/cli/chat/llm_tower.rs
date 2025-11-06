//! Tower-based LLM service stack with rate limiting

use tower::Service;
use tower::limit::ConcurrencyLimit;
use crate::api_client::ApiClient;
use super::llm_service::{LLMService, LLMRequest, RequestPriority};
use super::parser::{SendMessageStream, SendMessageError};

/// Tower-based LLM service with rate limiting
pub struct LLMTower {
    service: ConcurrencyLimit<LLMService>,
}

impl LLMTower {
    /// Create new Tower stack for LLM calls
    /// 
    /// # Arguments
    /// * `client` - API client for making LLM calls
    /// * `max_concurrent` - Maximum concurrent LLM API calls
    pub fn new(client: ApiClient, max_concurrent: usize) -> Self {
        let service = LLMService::new(client);
        let limited = ConcurrencyLimit::new(service, max_concurrent);
        
        Self { service: limited }
    }
    
    /// Make a high-priority LLM call (active session)
    /// Blocks until capacity available
    pub async fn call_high_priority(
        &mut self,
        conversation_state: crate::api_client::model::ConversationState,
    ) -> Result<SendMessageStream, SendMessageError> {
        use tower::ServiceExt;
        
        let req = LLMRequest {
            conversation_state,
            priority: RequestPriority::High,
        };
        
        self.service.ready().await?.call(req).await
    }
    
    /// Make a low-priority LLM call (background session)
    /// Blocks until capacity available
    pub async fn call_low_priority(
        &mut self,
        conversation_state: crate::api_client::model::ConversationState,
    ) -> Result<SendMessageStream, SendMessageError> {
        use tower::ServiceExt;
        
        let req = LLMRequest {
            conversation_state,
            priority: RequestPriority::Low,
        };
        
        self.service.ready().await?.call(req).await
    }
    
    /// Try to make a low-priority call without blocking
    /// Returns None if service not ready (at capacity)
    pub fn try_call_low_priority(
        &mut self,
        conversation_state: crate::api_client::model::ConversationState,
    ) -> Option<std::pin::Pin<Box<dyn std::future::Future<Output = Result<SendMessageStream, SendMessageError>> + Send>>> {
        use std::task::{Context, Poll};
        use std::pin::Pin;
        
        // Check if service is ready without blocking
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        
        match Pin::new(&mut self.service).poll_ready(&mut cx) {
            Poll::Ready(Ok(())) => {
                let req = LLMRequest {
                    conversation_state,
                    priority: RequestPriority::Low,
                };
                Some(Box::pin(self.service.call(req)))
            },
            _ => None,
        }
    }
}

impl Clone for LLMTower {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
        }
    }
}

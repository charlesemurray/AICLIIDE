use crate::{CortexError, Result};

/// Embedder wrapper for generating text embeddings
pub struct CortexEmbedder {
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    embedder: semantic_search_client::embedding::CandleTextEmbedder,
}

impl CortexEmbedder {
    /// Create a new embedder using Q CLI's CandleTextEmbedder
    pub fn new() -> Result<Self> {
        #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
        {
            let embedder = semantic_search_client::embedding::CandleTextEmbedder::new()
                .map_err(|e| CortexError::EmbeddingError(e.to_string()))?;
            Ok(Self { embedder })
        }
        
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            Err(CortexError::EmbeddingError(
                "Embeddings not supported on Linux ARM64".to_string()
            ))
        }
    }

    /// Generate embedding for a single text
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
        {
            self.embedder
                .embed(text)
                .map_err(|e| CortexError::EmbeddingError(e.to_string()))
        }
        
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            let _ = text;
            Err(CortexError::EmbeddingError(
                "Embeddings not supported on Linux ARM64".to_string()
            ))
        }
    }

    /// Get the dimensionality of embeddings (384 for all-MiniLM-L6-v2)
    pub fn dimensions(&self) -> usize {
        384
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    fn test_embedder_creation() {
        let embedder = CortexEmbedder::new();
        assert!(embedder.is_ok());
    }

    #[test]
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    fn test_embed_text() {
        let embedder = CortexEmbedder::new().unwrap();
        let embedding = embedder.embed("test text").unwrap();
        
        assert_eq!(embedding.len(), 384);
        assert!(embedding.iter().any(|&x| x != 0.0));
    }

    #[test]
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    fn test_dimensions() {
        let embedder = CortexEmbedder::new().unwrap();
        assert_eq!(embedder.dimensions(), 384);
    }
}

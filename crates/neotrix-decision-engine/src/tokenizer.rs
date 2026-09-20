//! Tokenizer wrapper for Laya model

use tokenizers::Tokenizer;
use crate::error::{Error, Result};

/// Tokenizer wrapper for Laya
pub struct LayaTokenizer {
    inner: Tokenizer,
    cls_token_id: u32,
    sep_token_id: u32,
    pad_token_id: u32,
    mask_token_id: u32,
}

impl LayaTokenizer {
    /// Create from file
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(path)
            .map_err(|e| Error::TokenizerError(e.to_string()))?;
        
        let cls_token_id = tokenizer.token_to_id("[CLS]")
            .ok_or_else(|| Error::TokenizerError("Missing [CLS] token".into()))?;
        let sep_token_id = tokenizer.token_to_id("[SEP]")
            .ok_or_else(|| Error::TokenizerError("Missing [SEP] token".into()))?;
        let pad_token_id = tokenizer.token_to_id("[PAD]")
            .unwrap_or(0);
        let mask_token_id = tokenizer.token_to_id("[MASK]")
            .unwrap_or(sep_token_id); // fallback to SEP if no MASK
        
        Ok(Self {
            inner: tokenizer,
            cls_token_id,
            sep_token_id,
            pad_token_id,
            mask_token_id,
        })
    }
    
    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let tokenizer = Tokenizer::from_bytes(bytes)
            .map_err(|e| Error::TokenizerError(e.to_string()))?;
        
        let cls_token_id = tokenizer.token_to_id("[CLS]")
            .ok_or_else(|| Error::TokenizerError("Missing [CLS] token".into()))?;
        let sep_token_id = tokenizer.token_to_id("[SEP]")
            .ok_or_else(|| Error::TokenizerError("Missing [SEP] token".into()))?;
        let pad_token_id = tokenizer.token_to_id("[PAD]")
            .unwrap_or(0);
        let mask_token_id = tokenizer.token_to_id("[MASK]")
            .unwrap_or(sep_token_id);
        
        Ok(Self {
            inner: tokenizer,
            cls_token_id,
            sep_token_id,
            pad_token_id,
            mask_token_id,
        })
    }
    
    /// Encode text to token IDs (with special tokens)
    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let encoding = self.inner.encode(text, true)
            .map_err(|e| Error::TokenizerError(e.to_string()))?;
        Ok(encoding.get_ids().to_vec())
    }
    
    /// Encode text without special tokens (used for building sequences manually)
    pub fn encode_without_special_tokens(&self, text: &str) -> Result<Vec<u32>> {
        let encoding = self.inner.encode(text, false)
            .map_err(|e| Error::TokenizerError(e.to_string()))?;
        Ok(encoding.get_ids().to_vec())
    }
    
    /// Decode token IDs to text
    pub fn decode(&self, tokens: &[u32]) -> Result<String> {
        self.inner.decode(tokens, false)
            .map_err(|e| Error::TokenizerError(e.to_string()))
    }
    
    /// Get CLS token ID
    pub fn cls_token_id(&self) -> u32 { self.cls_token_id }
    
    /// Get SEP token ID
    pub fn sep_token_id(&self) -> u32 { self.sep_token_id }
    
    /// Get PAD token ID
    pub fn pad_token_id(&self) -> u32 { self.pad_token_id }
    
    /// Get MASK token ID (used as decision point marker in upstream Laya)
    pub fn mask_token_id(&self) -> u32 { self.mask_token_id }
    
    /// Get vocab size
    pub fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn project_root() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.pop();
        path
    }

    fn try_load_tokenizer() -> Option<LayaTokenizer> {
        let root = project_root();
        let path = root.join("models/modernbert-large/tokenizer.json");
        if !path.exists() {
            return None;
        }
        LayaTokenizer::from_file(&path).ok()
    }

    #[test]
    fn test_tokenizer_load() {
        let tok = match try_load_tokenizer() {
            Some(t) => t,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };
        assert!(tok.vocab_size() > 0);
    }

    #[test]
    fn test_special_token_ids() {
        let tok = match try_load_tokenizer() {
            Some(t) => t,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };
        assert!(tok.cls_token_id() > 0);
        assert!(tok.sep_token_id() > 0);
        // pad can be 0
    }

    #[test]
    fn test_encode_simple() {
        let tok = match try_load_tokenizer() {
            Some(t) => t,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };
        let ids = tok.encode("hello world").unwrap();
        assert!(!ids.is_empty());
        assert!(ids.len() < 10); // "hello world" should be a few tokens
    }

    #[test]
    fn test_encode_empty() {
        let tok = match try_load_tokenizer() {
            Some(t) => t,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };
        let _ids = tok.encode("").unwrap();
        // Empty input may produce 0 tokens or just special tokens
        // Either is acceptable
    }

    #[test]
    fn test_decode_roundtrip() {
        let tok = match try_load_tokenizer() {
            Some(t) => t,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };
        let original = "server is down";
        // Encode with special tokens, then decode only the content tokens
        let ids = tok.encode(original).unwrap();
        let decoded = tok.decode(&ids).unwrap();
        // decode() returns the full sequence including special tokens
        // Verify the content is preserved
        assert!(decoded.contains(original));
        // Also test encode without special tokens
        let encoding = tok.inner.encode(original, false)
            .map_err(|e| Error::TokenizerError(e.to_string())).unwrap();
        let no_special_ids = encoding.get_ids().to_vec();
        let decoded_no_special = tok.decode(&no_special_ids).unwrap();
        assert_eq!(decoded_no_special, original);
    }

    #[test]
    fn test_encode_long_text() {
        let tok = match try_load_tokenizer() {
            Some(t) => t,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };
        let text = "The quick brown fox jumps over the lazy dog. ";
        let long_text = text.repeat(20);
        let ids = tok.encode(&long_text).unwrap();
        assert!(ids.len() > 100); // Should produce many tokens
    }

    #[test]
    fn test_encode_special_chars() {
        let tok = match try_load_tokenizer() {
            Some(t) => t,
            None => { eprintln!("Skipping: tokenizer not found"); return; }
        };
        let ids = tok.encode("hello@world.com <test> {key: value}").unwrap();
        assert!(!ids.is_empty());
    }
}

use std::sync::{Arc, RwLock};
use uuid::Uuid;
use crate::types::{SessionState, TokenPair};
use crate::error::{Error, Result};

pub struct SessionManager {
    session: Arc<RwLock<Option<SessionState>>>,
    tokens: Arc<RwLock<Option<TokenPair>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            session: Arc::new(RwLock::new(None)),
            tokens: Arc::new(RwLock::new(None)),
        }
    }
    
    pub fn set_session(&self, session_id: Uuid, session_key: [u8; 32]) -> Result<()> {
        let mut session_guard = self.session.write()
            .map_err(|e| Error::Session(format!("Failed to acquire session write lock: {}", e)))?;
        
        *session_guard = Some(SessionState {
            session_id,
            session_key,
        });
        
        Ok(())
    }
    
    pub fn get_session(&self) -> Result<Option<SessionState>> {
        let session_guard = self.session.read()
            .map_err(|e| Error::Session(format!("Failed to acquire session read lock: {}", e)))?;
        
        Ok(session_guard.clone())
    }
    
    pub fn clear_session(&self) -> Result<()> {
        let mut session_guard = self.session.write()
            .map_err(|e| Error::Session(format!("Failed to acquire session write lock: {}", e)))?;
        
        *session_guard = None;
        Ok(())
    }
    
    pub fn set_tokens(&self, access_token: String, refresh_token: Option<String>) -> Result<()> {
        let mut tokens_guard = self.tokens.write()
            .map_err(|e| Error::Authentication(format!("Failed to acquire tokens write lock: {}", e)))?;
        
        *tokens_guard = Some(TokenPair {
            access_token,
            refresh_token,
        });
        
        Ok(())
    }
    
    pub fn get_tokens(&self) -> Result<Option<TokenPair>> {
        let tokens_guard = self.tokens.read()
            .map_err(|e| Error::Authentication(format!("Failed to acquire tokens read lock: {}", e)))?;
        
        Ok(tokens_guard.clone())
    }
    
    pub fn get_access_token(&self) -> Result<Option<String>> {
        let tokens_guard = self.tokens.read()
            .map_err(|e| Error::Authentication(format!("Failed to acquire tokens read lock: {}", e)))?;
        
        Ok(tokens_guard.as_ref().map(|t| t.access_token.clone()))
    }
    
    pub fn get_refresh_token(&self) -> Result<Option<String>> {
        let tokens_guard = self.tokens.read()
            .map_err(|e| Error::Authentication(format!("Failed to acquire tokens read lock: {}", e)))?;
        
        Ok(tokens_guard.as_ref().and_then(|t| t.refresh_token.clone()))
    }
    
    pub fn update_access_token(&self, access_token: String) -> Result<()> {
        let mut tokens_guard = self.tokens.write()
            .map_err(|e| Error::Authentication(format!("Failed to acquire tokens write lock: {}", e)))?;
        
        if let Some(tokens) = tokens_guard.as_mut() {
            tokens.access_token = access_token;
            Ok(())
        } else {
            Err(Error::Authentication("No tokens to update".to_string()))
        }
    }
    
    pub fn clear_tokens(&self) -> Result<()> {
        let mut tokens_guard = self.tokens.write()
            .map_err(|e| Error::Authentication(format!("Failed to acquire tokens write lock: {}", e)))?;
        
        *tokens_guard = None;
        Ok(())
    }
    
    pub fn clear_all(&self) -> Result<()> {
        self.clear_session()?;
        self.clear_tokens()?;
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_management() {
        let manager = SessionManager::new();
        
        // Initially empty
        assert!(manager.get_session().unwrap().is_none());
        
        // Set session
        let session_id = Uuid::new_v4();
        let session_key = [0u8; 32];
        manager.set_session(session_id, session_key).unwrap();
        
        // Retrieve session
        let session = manager.get_session().unwrap().unwrap();
        assert_eq!(session.session_id, session_id);
        assert_eq!(session.session_key, session_key);
        
        // Clear session
        manager.clear_session().unwrap();
        assert!(manager.get_session().unwrap().is_none());
    }
    
    #[test]
    fn test_token_management() {
        let manager = SessionManager::new();
        
        // Initially empty
        assert!(manager.get_tokens().unwrap().is_none());
        
        // Set tokens
        manager.set_tokens("access".to_string(), Some("refresh".to_string())).unwrap();
        
        // Retrieve tokens
        let tokens = manager.get_tokens().unwrap().unwrap();
        assert_eq!(tokens.access_token, "access");
        assert_eq!(tokens.refresh_token, Some("refresh".to_string()));
        
        // Update access token
        manager.update_access_token("new_access".to_string()).unwrap();
        assert_eq!(manager.get_access_token().unwrap(), Some("new_access".to_string()));
        
        // Clear tokens
        manager.clear_tokens().unwrap();
        assert!(manager.get_tokens().unwrap().is_none());
    }
}
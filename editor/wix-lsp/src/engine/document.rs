//! Document management for the LSP engine
//!
//! Manages open documents in the LSP session with thread-safe access.

use dashmap::DashMap;
use tower_lsp::lsp_types::Url;

/// Manages open documents in the LSP session
#[derive(Debug, Default)]
pub struct DocumentManager {
    /// Map of document URI to content
    documents: DashMap<Url, DocumentState>,
}

/// State of an open document
#[derive(Debug, Clone)]
pub struct DocumentState {
    /// Document content
    pub content: String,
    /// Document version
    pub version: i32,
}

impl DocumentManager {
    /// Create a new document manager
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
        }
    }

    /// Open a document
    pub fn open(&self, uri: Url, content: String, version: i32) {
        self.documents
            .insert(uri, DocumentState { content, version });
    }

    /// Update a document (full content replacement)
    pub fn update(&self, uri: &Url, content: String, version: i32) {
        if let Some(mut doc) = self.documents.get_mut(uri) {
            doc.content = content;
            doc.version = version;
        }
    }

    /// Close a document
    pub fn close(&self, uri: &Url) {
        self.documents.remove(uri);
    }

    /// Get document content
    pub fn get_content(&self, uri: &Url) -> Option<String> {
        self.documents.get(uri).map(|doc| doc.content.clone())
    }

    /// Get document state
    pub fn get(&self, uri: &Url) -> Option<DocumentState> {
        self.documents.get(uri).map(|doc| doc.clone())
    }

    /// Check if document is open
    pub fn is_open(&self, uri: &Url) -> bool {
        self.documents.contains_key(uri)
    }

    /// Get number of open documents
    pub fn count(&self) -> usize {
        self.documents.len()
    }

    /// Get all document URIs
    pub fn uris(&self) -> Vec<Url> {
        self.documents.iter().map(|r| r.key().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_uri(path: &str) -> Url {
        Url::parse(&format!("file://{}", path)).unwrap()
    }

    #[test]
    fn test_document_manager_new() {
        let manager = DocumentManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_open_document() {
        let manager = DocumentManager::new();
        let uri = test_uri("/test.wxs");
        manager.open(uri.clone(), "content".to_string(), 1);

        assert!(manager.is_open(&uri));
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.get_content(&uri), Some("content".to_string()));
    }

    #[test]
    fn test_update_document() {
        let manager = DocumentManager::new();
        let uri = test_uri("/test.wxs");
        manager.open(uri.clone(), "old".to_string(), 1);
        manager.update(&uri, "new".to_string(), 2);

        let doc = manager.get(&uri).unwrap();
        assert_eq!(doc.content, "new");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_close_document() {
        let manager = DocumentManager::new();
        let uri = test_uri("/test.wxs");
        manager.open(uri.clone(), "content".to_string(), 1);
        manager.close(&uri);

        assert!(!manager.is_open(&uri));
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_get_nonexistent_document() {
        let manager = DocumentManager::new();
        let uri = test_uri("/nonexistent.wxs");

        assert!(manager.get_content(&uri).is_none());
        assert!(manager.get(&uri).is_none());
    }

    #[test]
    fn test_update_nonexistent_document() {
        let manager = DocumentManager::new();
        let uri = test_uri("/nonexistent.wxs");

        // Should not panic
        manager.update(&uri, "content".to_string(), 1);

        // Document should not exist
        assert!(!manager.is_open(&uri));
    }

    #[test]
    fn test_multiple_documents() {
        let manager = DocumentManager::new();
        let uri1 = test_uri("/test1.wxs");
        let uri2 = test_uri("/test2.wxs");

        manager.open(uri1.clone(), "content1".to_string(), 1);
        manager.open(uri2.clone(), "content2".to_string(), 1);

        assert_eq!(manager.count(), 2);
        assert_eq!(manager.get_content(&uri1), Some("content1".to_string()));
        assert_eq!(manager.get_content(&uri2), Some("content2".to_string()));
    }

    #[test]
    fn test_uris() {
        let manager = DocumentManager::new();
        let uri1 = test_uri("/test1.wxs");
        let uri2 = test_uri("/test2.wxs");

        manager.open(uri1.clone(), "content1".to_string(), 1);
        manager.open(uri2.clone(), "content2".to_string(), 1);

        let uris = manager.uris();
        assert_eq!(uris.len(), 2);
        assert!(uris.contains(&uri1));
        assert!(uris.contains(&uri2));
    }

    #[test]
    fn test_document_state_clone() {
        let state = DocumentState {
            content: "test".to_string(),
            version: 1,
        };
        let cloned = state.clone();
        assert_eq!(cloned.content, "test");
        assert_eq!(cloned.version, 1);
    }
}

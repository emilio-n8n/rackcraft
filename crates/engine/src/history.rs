//! Bounded undo/redo over whole-document snapshots.
//!
//! Snapshots are cheap relative to user actions and remove an entire class of
//! inverse-command bugs. The history is bounded so a long editing session cannot
//! grow without limit.

use crate::document::Document;

/// Default maximum number of undo steps retained.
pub const DEFAULT_LIMIT: usize = 100;

/// Document editor state with undo/redo.
#[derive(Debug, Clone)]
pub struct History {
    document: Document,
    undo: Vec<Document>,
    redo: Vec<Document>,
    limit: usize,
    dirty: bool,
}

impl History {
    /// Wrap a document with an empty history.
    #[must_use]
    pub fn new(document: Document) -> Self {
        Self {
            document,
            undo: Vec::new(),
            redo: Vec::new(),
            limit: DEFAULT_LIMIT,
            dirty: false,
        }
    }

    /// Wrap a document with a custom history depth.
    #[must_use]
    pub fn with_limit(document: Document, limit: usize) -> Self {
        let mut history = Self::new(document);
        history.limit = limit.max(1);
        history
    }

    /// Read the current document.
    #[must_use]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Apply a mutation, recording one undo step only when the document changes.
    ///
    /// Returns `true` if the document changed.
    pub fn apply(&mut self, mutate: impl FnOnce(&mut Document)) -> bool {
        let before = self.document.clone();
        mutate(&mut self.document);
        if self.document == before {
            return false;
        }
        self.undo.push(before);
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
        true
    }

    /// Replace the document and reset history (Open / New / sample).
    pub fn replace(&mut self, document: Document) {
        self.document = document;
        self.undo.clear();
        self.redo.clear();
        self.dirty = false;
    }

    /// Undo one step. Returns `true` if anything changed.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut self.document, previous);
        self.redo.push(current);
        self.dirty = true;
        true
    }

    /// Redo one step. Returns `true` if anything changed.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut self.document, next);
        self.undo.push(current);
        self.dirty = true;
        true
    }

    /// Whether undo is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether redo is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Depth of the undo stack.
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Mark the current document as saved.
    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    /// Whether there are unsaved changes.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Allocate a fresh item id matching the current document state.
    #[must_use]
    pub fn peek_next_id(&self) -> u64 {
        self.document.next_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Item, StencilRef};
    use crate::geometry::{Point, Size};

    fn document_named(name: &str) -> Document {
        Document::new(name)
    }

    #[test]
    fn mutation_records_one_step_and_undo_redo_restore() {
        let mut history = History::new(document_named("A"));
        assert!(!history.is_dirty());
        assert!(history.apply(|doc| doc.name = "B".into()));
        assert!(history.is_dirty());
        assert!(!history.apply(|doc| doc.name = "B".into()), "no-op ignored");
        assert_eq!(history.undo_depth(), 1);
        assert!(history.undo());
        assert_eq!(history.document().name, "A");
        assert!(history.can_redo());
        assert!(history.redo());
        assert_eq!(history.document().name, "B");
    }

    #[test]
    fn bound_is_respected_and_oldest_steps_are_dropped() {
        let mut history = History::with_limit(document_named("0"), 2);
        for name in ["1", "2", "3"] {
            history.apply(|doc| doc.name = name.into());
        }
        assert_eq!(history.undo_depth(), 2);
        assert_eq!(history.document().name, "3");
        assert!(history.undo());
        assert!(history.undo());
        assert_eq!(history.document().name, "1");
        assert!(!history.undo());
    }

    #[test]
    fn new_mutation_clears_redo() {
        let mut history = History::new(document_named("A"));
        history.apply(|doc| doc.name = "B".into());
        history.undo();
        assert!(history.can_redo());
        history.apply(|doc| doc.name = "C".into());
        assert!(!history.can_redo());
        assert_eq!(history.document().name, "C");
    }

    #[test]
    fn apply_can_add_items_and_mark_saved() {
        let mut history = History::new(document_named("Document"));
        history.apply(|doc| {
            doc.add_item(Item::new(
                StencilRef::default(),
                "Item",
                Point::default(),
                Size::new(1.0, 1.0),
            ));
        });
        assert_eq!(history.document().items.len(), 1);
        history.mark_saved();
        assert!(!history.is_dirty());
        history.undo();
        assert!(history.is_dirty());
        assert_eq!(history.document().items.len(), 0);
    }

    #[test]
    fn replace_resets_everything() {
        let mut history = History::new(document_named("A"));
        history.apply(|doc| doc.name = "B".into());
        history.replace(document_named("C"));
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert!(!history.is_dirty());
        assert_eq!(history.document().name, "C");
        assert_eq!(history.peek_next_id(), 1);
    }
}

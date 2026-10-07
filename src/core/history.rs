//! Sistema de Histórico (Undo / Redo) com controle de memória.
//! Traduzido de Compositor/Document/DocumentHistory.swift.

use crate::core::document::Document;

pub struct HistoryStep {
    pub description: String,
    pub snapshot: Document,
}

pub struct DocumentHistory {
    undo_stack: Vec<HistoryStep>,
    redo_stack: Vec<HistoryStep>,
    max_steps: usize,
}

impl DocumentHistory {
    pub fn new(max_steps: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_steps: if max_steps == 0 { 50 } else { max_steps },
        }
    }

    pub fn push(&mut self, description: String, current_state: &Document) {
        self.redo_stack.clear();
        self.undo_stack.push(HistoryStep {
            description,
            snapshot: current_state.clone(),
        });
        if self.undo_stack.len() > self.max_steps {
            self.undo_stack.remove(0);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self, current_state: &Document) -> Option<Document> {
        let step = self.undo_stack.pop()?;
        self.redo_stack.push(HistoryStep {
            description: step.description,
            snapshot: current_state.clone(),
        });
        Some(step.snapshot)
    }

    pub fn redo(&mut self, current_state: &Document) -> Option<Document> {
        let step = self.redo_stack.pop()?;
        self.undo_stack.push(HistoryStep {
            description: step.description,
            snapshot: current_state.clone(),
        });
        Some(step.snapshot)
    }
}

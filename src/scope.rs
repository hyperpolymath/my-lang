//! Symbol table and scope management for My Language
//!
//! Provides hierarchical scope management for name resolution.

use crate::types::Ty;
use crate::token::Span;
use std::collections::HashMap;

/// Information about a symbol (variable, function, type, etc.)
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub ty: Ty,
    pub span: Span,
    pub mutable: bool,
}

/// The kind of symbol
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
    Struct,
    Effect,
    AiModel,
    Prompt,
}

/// A single scope level
#[derive(Debug, Default)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    /// Parent scope index (None for global scope)
    parent: Option<usize>,
}

/// Hierarchical symbol table managing multiple scopes
#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current: usize,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    /// Create a new symbol table with a global scope
    pub fn new() -> Self {
        let mut table = Self {
            scopes: Vec::new(),
            current: 0,
        };
        // Create global scope
        table.scopes.push(Scope::default());
        table
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        let parent = Some(self.current);
        let new_scope = Scope {
            symbols: HashMap::new(),
            parent,
        };
        self.scopes.push(new_scope);
        self.current = self.scopes.len() - 1;
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current].parent {
            self.current = parent;
        }
    }

    /// Define a symbol in the current scope
    pub fn define(&mut self, symbol: Symbol) -> Result<(), String> {
        let name = symbol.name.clone();
        if self.scopes[self.current].symbols.contains_key(&name) {
            return Err(format!("Symbol '{}' is already defined in this scope", name));
        }
        self.scopes[self.current].symbols.insert(name, symbol);
        Ok(())
    }

    /// Look up a symbol by name, searching from current scope up to global
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut scope_idx = Some(self.current);

        while let Some(idx) = scope_idx {
            if let Some(symbol) = self.scopes[idx].symbols.get(name) {
                return Some(symbol);
            }
            scope_idx = self.scopes[idx].parent;
        }

        None
    }

    /// Look up a symbol only in the current scope
    pub fn lookup_current(&self, name: &str) -> Option<&Symbol> {
        self.scopes[self.current].symbols.get(name)
    }

    /// Check if a name is defined in any accessible scope
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Get all symbols in the current scope
    pub fn current_scope_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.scopes[self.current].symbols.values()
    }

    /// Get the current scope depth (0 = global)
    pub fn depth(&self) -> usize {
        let mut depth = 0;
        let mut scope_idx = Some(self.current);
        while let Some(idx) = scope_idx {
            if self.scopes[idx].parent.is_some() {
                depth += 1;
            }
            scope_idx = self.scopes[idx].parent;
        }
        depth
    }
}

/// Type environment for tracking type definitions
#[derive(Debug, Default)]
pub struct TypeEnv {
    /// Struct definitions: name -> (fields, type_params)
    structs: HashMap<String, StructDef>,
    /// Effect definitions: name -> operations
    effects: HashMap<String, EffectDef>,
    /// AI model definitions
    ai_models: HashMap<String, AiModelDef>,
    /// Prompt definitions
    prompts: HashMap<String, PromptDef>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Ty)>,
    pub type_params: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EffectDef {
    pub name: String,
    pub operations: Vec<(String, Ty)>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AiModelDef {
    pub name: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PromptDef {
    pub name: String,
    pub template: String,
    pub span: Span,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define_struct(&mut self, def: StructDef) -> Result<(), String> {
        if self.structs.contains_key(&def.name) {
            return Err(format!("Struct '{}' is already defined", def.name));
        }
        self.structs.insert(def.name.clone(), def);
        Ok(())
    }

    pub fn define_effect(&mut self, def: EffectDef) -> Result<(), String> {
        if self.effects.contains_key(&def.name) {
            return Err(format!("Effect '{}' is already defined", def.name));
        }
        self.effects.insert(def.name.clone(), def);
        Ok(())
    }

    pub fn define_ai_model(&mut self, def: AiModelDef) -> Result<(), String> {
        if self.ai_models.contains_key(&def.name) {
            return Err(format!("AI model '{}' is already defined", def.name));
        }
        self.ai_models.insert(def.name.clone(), def);
        Ok(())
    }

    pub fn define_prompt(&mut self, def: PromptDef) -> Result<(), String> {
        if self.prompts.contains_key(&def.name) {
            return Err(format!("Prompt '{}' is already defined", def.name));
        }
        self.prompts.insert(def.name.clone(), def);
        Ok(())
    }

    pub fn get_struct(&self, name: &str) -> Option<&StructDef> {
        self.structs.get(name)
    }

    pub fn get_effect(&self, name: &str) -> Option<&EffectDef> {
        self.effects.get(name)
    }

    pub fn get_ai_model(&self, name: &str) -> Option<&AiModelDef> {
        self.ai_models.get(name)
    }

    pub fn get_prompt(&self, name: &str) -> Option<&PromptDef> {
        self.prompts.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_basic() {
        let mut table = SymbolTable::new();

        let sym = Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            ty: Ty::Int,
            span: Span::default(),
            mutable: false,
        };

        table.define(sym).unwrap();
        assert!(table.is_defined("x"));
        assert!(!table.is_defined("y"));

        let found = table.lookup("x").unwrap();
        assert_eq!(found.ty, Ty::Int);
    }

    #[test]
    fn test_nested_scopes() {
        let mut table = SymbolTable::new();

        // Define in global
        table.define(Symbol {
            name: "global".to_string(),
            kind: SymbolKind::Variable,
            ty: Ty::Int,
            span: Span::default(),
            mutable: false,
        }).unwrap();

        // Enter new scope
        table.enter_scope();

        // Define local
        table.define(Symbol {
            name: "local".to_string(),
            kind: SymbolKind::Variable,
            ty: Ty::String,
            span: Span::default(),
            mutable: false,
        }).unwrap();

        // Can see both
        assert!(table.is_defined("global"));
        assert!(table.is_defined("local"));

        // Exit scope
        table.exit_scope();

        // Can only see global
        assert!(table.is_defined("global"));
        assert!(!table.is_defined("local"));
    }

    #[test]
    fn test_shadowing() {
        let mut table = SymbolTable::new();

        table.define(Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            ty: Ty::Int,
            span: Span::default(),
            mutable: false,
        }).unwrap();

        table.enter_scope();

        // Shadow x with different type
        table.define(Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            ty: Ty::String,
            span: Span::default(),
            mutable: false,
        }).unwrap();

        // Sees shadowed version
        let found = table.lookup("x").unwrap();
        assert_eq!(found.ty, Ty::String);

        table.exit_scope();

        // Sees original
        let found = table.lookup("x").unwrap();
        assert_eq!(found.ty, Ty::Int);
    }
}

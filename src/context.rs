#![allow(dead_code)]

use tree_sitter::Tree;

use crate::value::Value;
use std::collections::HashMap;

type SymbolTable = HashMap<String, Value>;

#[derive(Debug)]
pub struct Context<'a> {
  // pub env: HashMap<String, Value>,
  // pub scope_env: Option<HashMap<String, Value>>,
  pub call_stack: Vec<SymbolTable>,
  pub tree: &'a Tree,
}

impl<'a> Context<'a> {
  pub fn new(tree: &'a Tree) -> Context<'a> {
    let mut ctx = Context {
      call_stack: Vec::new(),
      tree,
    };

    // create global scope
    ctx.init_scope();

    return ctx;
  }

  pub fn depth(&self) -> usize {
    // returns the function depth of the context
    return self.call_stack.len();
  }

  pub fn search_in_stack(&mut self, varname: &String) -> Option<&mut Value> {
    // find the first entry from the top of the stack that matches the variable
    // name (lexical scoping)

    let reverse_iter = self.call_stack.iter_mut().rev();

    // we check through entire call stack before declaring none
    for table in reverse_iter {
      if !table.contains_key(varname) {
        continue;
      }

      return table.get_mut(varname);
    }

    return None;
  }

  // create a new scope for the call stack
  pub fn init_scope(&mut self) {
    let new_scope: SymbolTable = HashMap::new();

    self.call_stack.push(new_scope);
  }

  // destroy the topmost scope, popping it off the call stack
  pub fn destroy_scope(&mut self) {
    self.call_stack.pop();
  }

  pub fn current_scope(&mut self) -> &mut SymbolTable {
    return self.call_stack.last_mut().unwrap();
  }
}

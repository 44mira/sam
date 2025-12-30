#![allow(dead_code)]

use crate::value::Value;
use std::collections::{HashMap, hash_map::Entry};

type SymbolTable = HashMap<String, Value>;

#[derive(Debug)]
pub struct Context {
  // pub env: HashMap<String, Value>,
  // pub scope_env: Option<HashMap<String, Value>>,
  pub call_stack: Vec<SymbolTable>,
}

impl Context {
  pub fn new() -> Self {
    return Context {
      call_stack: Vec::new(),
    };
  }

  pub fn depth(&self) -> usize {
    // returns the function depth of the context
    return self.call_stack.len();
  }

  pub fn search_in_stack(
    &mut self,
    varname: &String,
  ) -> Option<Entry<String, Value>> {
    // find the first entry from the top of the stack that matches the variable
    // name (lexical scoping)

    let reverse_iter = self.call_stack.iter_mut().rev();

    for table in reverse_iter {
      if !table.contains_key(varname) {
        continue;
      }

      return Some(table.entry(varname.to_owned()));
    }

    return None;
  }
}

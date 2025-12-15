#![allow(dead_code)]

use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context {
  pub env: HashMap<String, Value>,
  pub call_stack: Vec<String>,
}

impl Context {
  pub fn new() -> Self {
    return Context {
      env: HashMap::new(),
      call_stack: Vec::new(),
    };
  }

  pub fn depth(&self) -> usize {
    // returns the function depth of the context
    return self.call_stack.len();
  }
}

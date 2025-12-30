#![allow(dead_code)]

use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context {
  pub env: HashMap<String, Value>,
  pub scope_env: Option<HashMap<String, Value>>,

  pub call_stack: Vec<String>,
}

impl Context {
  pub fn new() -> Self {
    return Context {
      env: HashMap::new(),
      scope_env: None,
      call_stack: Vec::new(),
    };
  }

  pub fn depth(&self) -> usize {
    // returns the function depth of the context
    return self.call_stack.len();
  }

  pub fn init_scope(&mut self) {
    // only initialize when scope_env is None
    if !self.scope_env.is_none() {
      return;
    }

    let scope_env = HashMap::new();

    self.scope_env = Some(scope_env);
  }
}

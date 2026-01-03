#![allow(dead_code)]

use std::ops::{Range, *};

use tree_sitter::Node;

use crate::{
  context::{Context, EvalControl},
  evaluate::evaluate_expression,
};

// TODO: Add string and functions
#[derive(Debug, Clone)]
pub enum Value {
  SamNumber(Number),
  // byte range of function for lazy evaluation
  SamFunction(Function),
  Undefined,
}

#[derive(Debug, Clone)]
pub struct Function {
  // functions are represented as their byte range and parameter list
  pub body: Range<usize>,
  pub params: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum Number {
  SamInt(i64),
  SamFloat(f64),
}

/* =========================
Function internal representation
========================= */

impl Function {
  pub fn new(body: Range<usize>, params: Vec<String>) -> Self {
    return Function { body, params };
  }

  pub fn extract_params(
    node: Node,
    source: &[u8],
  ) -> Result<Vec<String>, String> {
    let mut params = Vec::new();
    let mut walker = node.walk();

    for child in node.named_children(&mut walker) {
      if child.kind() == "identifier" {
        let Ok(varname) = child.utf8_text(source) else {
          return Err(format!(
            "There was an error when parsing the variable name of a parameter."
          ));
        };

        params.push(varname.to_owned());
      }
    }

    Ok(params)
  }

  pub fn extract_args(
    node: Node,
    ctx: &mut Context,
    source: &[u8],
  ) -> Result<Vec<Value>, String> {
    let mut args = Vec::new();
    let mut walker = node.walk();

    for arg in node.named_children(&mut walker) {
      let EvalControl::Value(val) = evaluate_expression(arg, ctx, source)?
      else {
        return Err(format!(
          "Unexpected return expression. {:#?}",
          node.range()
        ));
      };
      args.push(val);
    }

    Ok(args)
  }
}

/* =========================
Number arithmetic
========================= */

impl Number {
  fn as_f64(self) -> f64 {
    match self {
      Number::SamInt(i) => i as f64,
      Number::SamFloat(f) => f,
    }
  }
}

impl Add for Number {
  type Output = Number;

  fn add(self, rhs: Number) -> Number {
    match (self, rhs) {
      (Number::SamInt(a), Number::SamInt(b)) => Number::SamInt(a + b),
      (a, b) => Number::SamFloat(a.as_f64() + b.as_f64()),
    }
  }
}

impl Sub for Number {
  type Output = Number;

  fn sub(self, rhs: Number) -> Number {
    match (self, rhs) {
      (Number::SamInt(a), Number::SamInt(b)) => Number::SamInt(a - b),
      (a, b) => Number::SamFloat(a.as_f64() - b.as_f64()),
    }
  }
}

impl Mul for Number {
  type Output = Number;

  fn mul(self, rhs: Number) -> Number {
    match (self, rhs) {
      (Number::SamInt(a), Number::SamInt(b)) => Number::SamInt(a * b),
      (a, b) => Number::SamFloat(a.as_f64() * b.as_f64()),
    }
  }
}

impl Div for Number {
  type Output = Number;

  fn div(self, rhs: Number) -> Number {
    Number::SamFloat(self.as_f64() / rhs.as_f64())
  }
}

/* =========================
Value arithmetic
========================= */

impl Add for Value {
  type Output = Value;

  fn add(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::SamNumber(a), Value::SamNumber(b)) => Value::SamNumber(a + b),
      _ => Value::Undefined,
    }
  }
}

impl Sub for Value {
  type Output = Value;

  fn sub(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::SamNumber(a), Value::SamNumber(b)) => Value::SamNumber(a - b),
      _ => Value::Undefined,
    }
  }
}

impl Mul for Value {
  type Output = Value;

  fn mul(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::SamNumber(a), Value::SamNumber(b)) => Value::SamNumber(a * b),
      _ => Value::Undefined,
    }
  }
}

impl Div for Value {
  type Output = Value;

  fn div(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::SamNumber(a), Value::SamNumber(b)) => Value::SamNumber(a / b),
      _ => Value::Undefined,
    }
  }
}

/* =========================
Number modulo
========================= */

impl Rem for Number {
  type Output = Number;

  fn rem(self, rhs: Number) -> Number {
    match (self, rhs) {
      (Number::SamInt(a), Number::SamInt(b)) => Number::SamInt(a % b),
      (a, b) => Number::SamFloat(a.as_f64().rem_euclid(b.as_f64())),
    }
  }
}

/* =========================
Value modulo
========================= */

impl Rem for Value {
  type Output = Value;

  fn rem(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::SamNumber(a), Value::SamNumber(b)) => {
        // Explicit zero check
        match b {
          Number::SamInt(0) => Value::Undefined,
          Number::SamFloat(f) if f == 0.0 => Value::Undefined,
          _ => Value::SamNumber(a % b),
        }
      }
      _ => Value::Undefined,
    }
  }
}

// We are limited to only using PartialEq and PartialOrd due to utilizing f64
// as the internal value.

/* =========================
From helper conversions
========================= */

impl From<bool> for Value {
  fn from(b: bool) -> Self {
    Value::SamNumber(Number::SamInt(if b { 1 } else { 0 }))
  }
}

/* =========================
Number comparison
========================= */

impl PartialEq for Number {
  fn eq(&self, other: &Self) -> bool {
    self.as_f64() == other.as_f64()
  }
}

impl PartialOrd for Number {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.as_f64().partial_cmp(&other.as_f64())
  }
}

/* =========================
Value comparison
========================= */

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Value::SamNumber(a), Value::SamNumber(b)) => a == b,
      (Value::Undefined, Value::Undefined) => true,
      _ => false,
    }
  }
}

impl PartialOrd for Value {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    match (self, other) {
      (Value::SamNumber(a), Value::SamNumber(b)) => a.partial_cmp(b),
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /* =========================
     Number arithmetic
  ========================= */

  #[test]
  fn test_number_add_int() {
    let a = Number::SamInt(2);
    let b = Number::SamInt(3);
    assert_eq!(a + b, Number::SamInt(5));
  }

  #[test]
  fn test_number_add_float() {
    let a = Number::SamInt(2);
    let b = Number::SamFloat(0.5);
    assert_eq!(a + b, Number::SamFloat(2.5));
  }

  #[test]
  fn test_number_mul() {
    let a = Number::SamInt(4);
    let b = Number::SamInt(5);
    assert_eq!(a * b, Number::SamInt(20));
  }

  #[test]
  fn test_number_div() {
    let a = Number::SamInt(5);
    let b = Number::SamInt(2);
    assert_eq!(a / b, Number::SamFloat(2.5));
  }

  #[test]
  fn test_number_rem() {
    let a = Number::SamInt(7);
    let b = Number::SamInt(4);
    assert_eq!(a % b, Number::SamInt(3));
  }

  /* =========================
     Value arithmetic
  ========================= */

  #[test]
  fn test_value_add() {
    let a = Value::SamNumber(Number::SamInt(1));
    let b = Value::SamNumber(Number::SamInt(2));
    assert_eq!(a + b, Value::SamNumber(Number::SamInt(3)));
  }

  #[test]
  fn test_value_add_invalid() {
    let a = Value::Undefined;
    let b = Value::SamNumber(Number::SamInt(2));
    assert_eq!(a + b, Value::Undefined);
  }

  #[test]
  fn test_value_rem_div_by_zero() {
    let a = Value::SamNumber(Number::SamInt(5));
    let b = Value::SamNumber(Number::SamInt(0));
    assert_eq!(a % b, Value::Undefined);
  }

  /* =========================
     Comparisons
  ========================= */

  #[test]
  fn test_value_eq() {
    let a = Value::SamNumber(Number::SamInt(3));
    let b = Value::SamNumber(Number::SamFloat(3.0));
    assert_eq!(a, b);
  }

  #[test]
  fn test_value_ord() {
    let a = Value::SamNumber(Number::SamInt(1));
    let b = Value::SamNumber(Number::SamInt(2));
    assert!(a < b);
  }

  #[test]
  fn test_bool_into_value() {
    let v: Value = true.into();
    assert_eq!(v, Value::SamNumber(Number::SamInt(1)));
  }
}

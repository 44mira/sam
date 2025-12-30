#![allow(dead_code)]

use std::ops::*;

// TODO: Add string and functions
#[derive(Debug, Clone, Copy)]
pub enum Value {
  SamNumber(Number),
  Undefined,
}

#[derive(Debug, Clone, Copy)]
pub enum Number {
  SamInt(i64),
  SamFloat(f64),
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

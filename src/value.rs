#![allow(dead_code)]

// TODO: add string and functions
#[derive(Debug)]
pub enum Value {
  SamNumber(Number),
  Undefined,
}

#[derive(Debug)]
pub enum Number {
  SamInt(i64),
  SamFloat(f64),
}

#![allow(dead_code)]

use crate::value::{Number, Value};
use std::collections::HashMap;
use std::process::Command;

pub struct Shell;

impl Shell {
  pub fn call(name: &str, args: Vec<Value>) -> Result<Value, String> {
    // fallback shell call
    let mut cmd = Command::new(name);

    for arg in args {
      cmd.arg(arg.to_string());
    }

    let output = cmd.output().map_err(|e| e.to_string())?;

    // return obj
    let mut obj = HashMap::new();

    obj.insert(
      "stdout".to_string(),
      Value::SamString(String::from_utf8_lossy(&output.stdout).to_string()),
    );

    obj.insert(
      "stderr".to_string(),
      Value::SamString(String::from_utf8_lossy(&output.stderr).to_string()),
    );

    obj.insert(
      "status".to_string(),
      Value::SamNumber(Number::SamInt(
        output.status.code().unwrap_or(-1) as i64
      )),
    );

    return Ok(Value::SamObject(obj));
  }
}

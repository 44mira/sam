#![allow(dead_code)]

use crate::context::Context;
use crate::value::{ForeignFunction, Number, Value};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

pub struct Shell;
pub struct FFI;

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

impl FFI {
  pub fn register_ffi(
    path: &str,
    name: &str,
    ctx: &mut Context,
  ) -> Result<(), String> {
    let Ok(contents) = fs::read_to_string(&path) else {
      return Err(format!("There was an error in reading from {}.", path));
    };

    let Ok(json): Result<serde_json::Value, _> =
      serde_json::from_str(&contents)
    else {
      return Err(format!(
        "There was an error in parsing {} from {}.",
        name, path
      ));
    };

    let cmd = json
      .get(&name)
      .and_then(|v| v.as_str())
      .ok_or("Interface entry must be a string")?;

    ctx.current_scope().insert(
      name.to_owned(),
      Value::SamForeignFunction(ForeignFunction::new(cmd.to_owned())),
    );

    return Ok(());
  }

  pub fn call(f: &ForeignFunction, args: &Vec<Value>) -> Result<Value, String> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c");

    let full_cmd = format!(
      "{} {}",
      f.cmd,
      args
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ")
    );

    cmd.arg(full_cmd);

    let output = cmd.output().map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let Ok(parsed): Result<serde_json::Value, _> =
      serde_json::from_str(&stdout)
    else {
      return Err(format!(
        "There was an error in parsing the output of `{}`.",
        f.cmd
      ));
    };

    return Self::json_to_value(parsed);
  }

  pub fn json_to_value(v: serde_json::Value) -> Result<Value, String> {
    match v {
      serde_json::Value::Null => Ok(Value::Undefined),
      serde_json::Value::Bool(b) => {
        Ok(Value::SamNumber(Number::SamInt((b as i32).into())))
      }
      serde_json::Value::String(s) => Ok(Value::SamString(s)),
      serde_json::Value::Array(_a) => todo!(), // TODO: Arrays
      serde_json::Value::Object(o) => {
        let map = o
          .into_iter()
          .map(|(k, v)| Ok((k, Self::json_to_value(v)?)))
          .collect::<Result<_, String>>()?;

        Ok(Value::SamObject(map))
      }
      serde_json::Value::Number(n) => {
        let parsed = if let Some(i) = n.as_i64() {
          Ok(Number::SamInt(i))
        } else if let Some(f) = n.as_f64() {
          Ok(Number::SamFloat(f))
        } else {
          Err(format!("Invalid JSON number encountered."))
        };

        Ok(Value::SamNumber(parsed?))
      }
    }
  }
}

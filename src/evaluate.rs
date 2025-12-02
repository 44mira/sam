#![allow(dead_code)]

use std::collections::HashMap;
use tree_sitter::Node;

// TODO: add string and functions
#[derive(Debug)]
enum Value {
  SamNumber(Number),
  Undefined,
}

#[derive(Debug)]
enum Number {
  SamInt(i64),
  SamFloat(f64),
}

pub fn evaluate(root: &Node, source: &[u8]) -> Result<String, String> {
  if root.kind() != "source_file" {
    return Err(format!(
      "Source file node expected but not found. {:#?}",
      root.range()
    ));
  }

  // the variable table/environment, to be passed around as mutable reference
  let mut env: HashMap<String, Value> = HashMap::new();

  // TODO: handle interface
  let mut walker = root.walk();
  for child in root.named_children(&mut walker) {
    evaluate_statement(child, &mut env, source)?;
  }

  println!("{:#?}", env);

  return Ok("Evaluation successful".to_owned());
}

fn evaluate_statement(
  node: Node,
  env: &mut HashMap<String, Value>,
  source: &[u8],
) -> Result<(), String> {
  // TODO: add other statement types
  match node.kind() {
    "expression_statement" => {
      evaluate_expression(node.child(0).unwrap(), source)?;
    }
    "variable_declaration" => {
      evaluate_variable_declaration(node, env, source)?;
    }
    _ => {
      return Err(format!(
        "Unknown statement encountered. {:#?}",
        node.range()
      ));
    }
  }

  return Ok(());
}

fn evaluate_expression(node: Node, source: &[u8]) -> Result<Value, String> {
  // TODO: add other expression types
  return match node.kind() {
    "literal" => evaluate_literal(node, source),
    _ => Err(format!(
      "Unknown expression encountered. {:#?}",
      node.range()
    )),
  };
}

fn evaluate_variable_declaration(
  node: Node,
  env: &mut HashMap<String, Value>,
  source: &[u8],
) -> Result<(), String> {
  if node.kind() != "variable_declaration" {
    return Err(format!(
      "Variable declaration node expected but not found. {:#?}",
      node.range()
    ));
  }

  let mut walker = node.walk();
  for declarator in node.named_children(&mut walker) {
    evaluate_variable_declarator(declarator, env, source)?;
  }

  return Ok(());
}

fn evaluate_variable_declarator(
  node: Node,
  env: &mut HashMap<String, Value>,
  source: &[u8],
) -> Result<(), String> {
  if node.kind() != "variable_declarator" {
    return Err(format!(
      "Variable declarator expected but not found. {:#?}",
      node.range()
    ));
  }

  // obtain fields
  let ident =
    evaluate_identifier(node.child_by_field_name("variable").unwrap(), source)?;

  // create var in env and optionally set value
  env.insert(ident.to_owned(), Value::Undefined);

  if let Some(value) = node.child_by_field_name("value") {
    let v = evaluate_expression(value, source)?;
    env.entry(ident).insert_entry(v);
  }

  return Ok(());
}

fn evaluate_identifier(node: Node, source: &[u8]) -> Result<String, String> {
  if node.kind() != "identifier" {
    return Err(format!(
      "Identifier node expected but not found. {:#?}",
      node.range()
    ));
  }

  // get identifier name
  let ident = node.utf8_text(source).unwrap().to_owned();

  return Ok(ident);
}

fn evaluate_literal(node: Node, source: &[u8]) -> Result<Value, String> {
  if node.kind() != "literal" {
    return Err(format!(
      "Literal node expected but not found. {:#?}",
      node.range()
    ));
  }

  let value = node.child(0).unwrap();

  let result: Value;
  // TODO: handle string
  match value.kind() {
    "number" => {
      result = Value::SamNumber(evaluate_number(value, source)?);
    }
    _ => {
      return Err(format!(
        "Unknown literal type encountered. {:#?}",
        node.range()
      ));
    }
  }

  return Ok(result);
}

fn evaluate_number(node: Node, source: &[u8]) -> Result<Number, String> {
  if node.kind() != "number" {
    return Err(format!(
      "Number node expected but not found. {:#?}",
      node.range()
    ));
  }

  let value = node.utf8_text(source).unwrap();
  let parsed: Number;

  if value.contains(".") {
    parsed = Number::SamFloat(value.parse().unwrap());
  } else {
    parsed = Number::SamInt(value.parse().unwrap())
  }

  return Ok(parsed);
}

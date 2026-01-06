#![allow(dead_code)]

use crate::context::{Context, EvalControl, EvalResult};
use crate::ffi::{FFI, Shell};
use crate::value::{ForeignFunction, Function, Number, Value};
use tree_sitter::{Node, Tree};

fn expect_node(
  node: &Node,
  node_name: &str,
  message: &str,
) -> Result<(), String> {
  if node.kind() != node_name {
    return Err(format!("{} {:#?}", message, node.range()));
  }
  Ok(())
}

pub fn evaluate<'a>(
  root: &'a Node,
  source: &[u8],
  tree: &'a Tree,
) -> Result<Context<'a>, String> {
  expect_node(root, "source_file", "Expected source file")?;

  let mut ctx = Context::new(tree);

  let mut walker = root.walk();
  let mut children = root.named_children(&mut walker);

  // optionally check if the first is interfaces
  if let Some(first) = children.next() {
    if first.kind() == "interfaces" {
      evaluate_interfaces(first, &mut ctx, source)?;
    } else {
      evaluate_statement(first, &mut ctx, source)?;
    }
  }

  // run the rest as regular
  for child in children {
    match evaluate_statement(child, &mut ctx, source)? {
      EvalControl::Value(_) => {}
      EvalControl::Return(_) => {
        return Err("Return outside function".to_owned());
      }
    }
  }

  Ok(ctx)
}

/* =========================
Interfaces
========================= */

fn evaluate_interfaces(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  expect_node(&node, "interfaces", "Expected interfaces")?;

  let mut walker = node.walk();
  let interfaces = node.named_children(&mut walker);

  for interface in interfaces {
    evaluate_interface(interface, ctx, source)?;
  }

  return Ok(());
}

fn evaluate_interface(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  expect_node(&node, "interface", "Expected interface")?;

  let path =
    evaluate_string(node.child_by_field_name("path").unwrap(), source)?;
  let module =
    evaluate_identifier(node.child_by_field_name("module").unwrap(), source)?;

  FFI::register_ffi(&path, &module, ctx)?;

  return Ok(());
}

/* =========================
Statements
========================= */

fn evaluate_statement(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> EvalResult {
  match node.kind() {
    "expression_statement" => {
      let v = evaluate_expression(node.child(0).unwrap(), ctx, source)?;
      Ok(v)
    }

    "variable_declaration" => {
      evaluate_variable_declaration(node, ctx, source)?;
      Ok(EvalControl::Value(Value::Undefined))
    }

    "assignment" => {
      let v = evaluate_assignment(node, ctx, source)?;
      Ok(EvalControl::Value(v))
    }

    "return_statement" => evaluate_return_statement(node, ctx, source),

    _ => Err(format!("Unknown statement {:?}", node.range())),
  }
}

/* =========================
Expressions
========================= */

pub fn evaluate_expression(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> EvalResult {
  match node.kind() {
    "literal" => Ok(EvalControl::Value(evaluate_literal(node, source)?)),

    "binary_expression" => {
      let v = evaluate_binary_expression(node, ctx, source)?;
      Ok(EvalControl::Value(v))
    }

    "if_expression" => evaluate_if_expression(node, ctx, source),

    "lambda_expression" => {
      let v = evaluate_lambda_expression(node, ctx, source)?;
      Ok(EvalControl::Value(v))
    }

    "call_expression" => evaluate_call_expression(node, ctx, source),

    "identifier" => {
      let name = evaluate_identifier(node, source)?;
      let Some(var) = ctx.search_in_stack(&name) else {
        return Err(format!(
          "Variable {} not defined {:?}",
          name,
          node.range()
        ));
      };
      Ok(EvalControl::Value(var.clone()))
    }

    "nested_identifier" => evaluate_nested_identifier(node, ctx, source),

    "array_expression" => {
      let v = evaluate_array_expression(node, ctx, source)?;
      Ok(EvalControl::Value(v))
    }

    _ => Err(format!("Unknown expression {:?}", node.range())),
  }
}

/* =========================
Binary expression
========================= */

fn evaluate_binary_expression(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  expect_node(&node, "binary_expression", "Expected binary expression")?;

  let left = evaluate_expression(
    node.child_by_field_name("left").unwrap(),
    ctx,
    source,
  )?
  .to_value();

  let right = evaluate_expression(
    node.child_by_field_name("right").unwrap(),
    ctx,
    source,
  )?
  .to_value();

  let op = node.child(1).unwrap().utf8_text(source).unwrap().trim();

  Ok(match op {
    "+" => left + right,
    "-" => left - right,
    "*" => left * right,
    "/" => left / right,
    "%" => left % right,
    "<" => (left < right).into(),
    ">" => (left > right).into(),
    "==" => (left == right).into(),
    "<=" => (left <= right).into(),
    ">=" => (left >= right).into(),
    "!=" => (left != right).into(),
    "&&" => (left.into() && right.into()).into(),
    "||" => (left.into() || right.into()).into(),
    _ => return Err(format!("Unknown operator {:?}", node.range())),
  })
}

/* =========================
Variable declaration
========================= */

fn evaluate_variable_declaration(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  expect_node(&node, "variable_declaration", "Expected declaration")?;

  let mut walker = node.walk();
  for declarator in node.named_children(&mut walker) {
    evaluate_variable_declarator(declarator, ctx, source)?;
  }

  Ok(())
}

fn evaluate_variable_declarator(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  expect_node(&node, "variable_declarator", "Expected declarator")?;

  let ident =
    evaluate_identifier(node.child_by_field_name("variable").unwrap(), source)?;

  let value = node
    .child_by_field_name("value")
    .map(|n| evaluate_expression(n, ctx, source))
    .transpose()?
    .map(|v| v.to_value());

  let scope = ctx.current_scope();
  let entry = scope.entry(ident).or_insert(Value::Undefined);

  if let Some(v) = value {
    *entry = v;
  }

  Ok(())
}

/* =========================
Assignment
========================= */

fn evaluate_assignment(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  expect_node(&node, "assignment", "Expected assignment")?;

  let lhs =
    evaluate_identifier(node.child_by_field_name("lhs").unwrap(), source)?;

  let rhs =
    evaluate_expression(node.child_by_field_name("rhs").unwrap(), ctx, source)?
      .to_value();

  let Some(var) = ctx.search_in_stack(&lhs) else {
    return Err(format!(
      "Assigning to undefined variable {:?}",
      node.range()
    ));
  };

  *var = rhs.clone();
  Ok(rhs)
}

/* =========================
Attribute access
========================= */

fn evaluate_nested_identifier(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> EvalResult {
  let parent_node = node
    .child_by_field_name("parent")
    .ok_or("Missing parent in nested_identifier")?;

  let name_node = node
    .child_by_field_name("name")
    .ok_or("Missing name in nested_identifier")?;

  let parent_value = evaluate_expression(parent_node, ctx, source)?.to_value();

  let key = name_node.utf8_text(source).map_err(|e| e.to_string())?;

  return Ok(EvalControl::Value(parent_value.get_attr(&node, key)?));
}

/* =========================
If expression
========================= */

fn evaluate_if_expression(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> EvalResult {
  use {Number::SamInt, Value::SamNumber};

  expect_node(&node, "if_expression", "Expected if expression")?;

  let cond = evaluate_expression(
    node.child_by_field_name("condition").unwrap(),
    ctx,
    source,
  )?
  .to_value();

  let SamNumber(SamInt(c)) = cond else {
    return Err(format!("Condition must be integer {:?}", node.range()));
  };

  if c != 0 {
    return evaluate_statement_block(
      node.child_by_field_name("consequence").unwrap(),
      ctx,
      source,
      None,
    );
  }

  if let Some(else_arm) = node.child_by_field_name("else") {
    return match else_arm.kind() {
      "statement_block" => {
        evaluate_statement_block(else_arm, ctx, source, None)
      }
      "if_expression" => evaluate_if_expression(else_arm, ctx, source),
      _ => Err(format!("Invalid else {:?}", else_arm.range())),
    };
  }

  Ok(EvalControl::Value(Value::Undefined))
}

/* =========================
Lambda & Call
========================= */

fn evaluate_lambda_expression(
  node: Node,
  _ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  expect_node(&node, "lambda_expression", "Expected lambda")?;

  // retrieve byte representation for lazy evaluation
  let range = node.child_by_field_name("body").unwrap().byte_range();

  // temporarily represent as empty small Vec
  let mut params = Vec::with_capacity(1);

  // if parameters exist, replace the Vec
  if let Some(params_node) = node.child_by_field_name("parameters") {
    params = Function::extract_params(params_node, source)?;
  }

  return Ok(Value::SamFunction(Function::new(range, params)));
}

fn evaluate_call_expression(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> EvalResult {
  expect_node(&node, "call_expression", "Expected call")?;

  let func_node = node.child_by_field_name("function").unwrap();

  // temporarily represent as empty small Vec
  let mut args = Vec::with_capacity(1);

  if let Some(args_node) = node.child_by_field_name("arguments") {
    args = Function::extract_args(args_node, ctx, source)?;
  }

  if let Ok(EvalControl::Value(Value::SamFunction(func))) =
    evaluate_expression(func_node, ctx, source)
  {
    if args.len() != func.params.len() {
      return Err(format!("Argument count mismatch {:?}", node.range()));
    }

    let bindings = func.params.iter().cloned().zip(args).collect();

    let body = ctx
      .tree
      .root_node()
      .descendant_for_byte_range(func.body.start, func.body.end)
      .ok_or("Function body not found")?;

    return evaluate_statement_block(body, ctx, source, Some(bindings));
  }

  // Otherwise: shell fallback
  let command_name = match func_node.kind() {
    "identifier" => evaluate_identifier(func_node, source)?,
    _ => return Err(format!("Invalid shell command {:?}", func_node.range())),
  };

  let result;

  // check for FFI or Shell command
  if let Some(Value::SamForeignFunction(ff)) =
    ctx.global_scope().get(&command_name)
  {
    result = FFI::call(ff, &args)?;
  } else {
    result = Shell::call(&command_name, args)?;
  }

  return Ok(EvalControl::Value(result));
}

/* =========================
Statement block
========================= */

fn evaluate_statement_block(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
  bindings: Option<Vec<(String, Value)>>,
) -> EvalResult {
  expect_node(&node, "statement_block", "Expected block")?;

  ctx.init_scope();

  if let Some(bindings) = bindings {
    let scope = ctx.current_scope();
    for (name, value) in bindings {
      scope.insert(name, value);
    }
  }

  let mut walker = node.walk();
  for stmt in node.named_children(&mut walker) {
    match evaluate_statement(stmt, ctx, source)? {
      EvalControl::Value(_) => {}
      EvalControl::Return(v) => {
        ctx.destroy_scope();
        return Ok(EvalControl::Return(v));
      }
    }
  }

  ctx.destroy_scope();
  Ok(EvalControl::Value(Value::Undefined))
}

/* =========================
Return
========================= */

fn evaluate_return_statement(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> EvalResult {
  expect_node(&node, "return_statement", "Expected return")?;

  let value = match node.child_by_field_name("value") {
    Some(v) => evaluate_expression(v, ctx, source)?.to_value(),
    None => Value::Undefined,
  };

  Ok(EvalControl::Return(value))
}

/* =========================
Literals & identifiers
========================= */

fn evaluate_identifier(node: Node, source: &[u8]) -> Result<String, String> {
  expect_node(&node, "identifier", "Expected identifier")?;
  Ok(node.utf8_text(source).unwrap().to_owned())
}

fn evaluate_literal(node: Node, source: &[u8]) -> Result<Value, String> {
  expect_node(&node, "literal", "Expected literal")?;
  let child = node.child(0).unwrap();

  match child.kind() {
    "number" => Ok(Value::SamNumber(evaluate_number(child, source)?)),
    "string" => Ok(Value::SamString(evaluate_string(child, source)?)),
    _ => Err(format!("Unknown literal {:?}", node.range())),
  }
}

fn evaluate_string(node: Node, source: &[u8]) -> Result<String, String> {
  expect_node(&node, "string", "Expected string")?;

  let mut result = String::new();
  let mut walker = node.walk();

  for child in node.named_children(&mut walker) {
    match child.kind() {
      "string_fragment" => {
        result.push_str(child.utf8_text(source).unwrap());
      }
      "escape_sequence" => {
        let esc = child.utf8_text(source).unwrap();
        result.push(Value::decode_escape(esc)?);
      }
      _ => {}
    }
  }

  return Ok(result);
}

fn evaluate_number(node: Node, source: &[u8]) -> Result<Number, String> {
  expect_node(&node, "number", "Expected number")?;

  let text = node.utf8_text(source).unwrap();
  if text.contains('.') {
    Ok(Number::SamFloat(text.parse().unwrap()))
  } else {
    Ok(Number::SamInt(text.parse().unwrap()))
  }
}

/* =========================
Arrays
========================= */

fn evaluate_array_expression(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  expect_node(&node, "array_expression", "Expected array expression")?;

  let mut walker = node.walk();

  let mut arr = Vec::new();

  // iterate over items in list
  for item in node.named_children(&mut walker) {
    let EvalControl::Value(val) = evaluate_expression(item, ctx, source)?
    else {
      return Err(format!("Unexpected return statement. {:#?}", item.range()));
    };

    arr.push(val);
  }

  return Ok(Value::SamArray(arr));
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tree_sitter::{Language, Parser};

  // retrieve Language struct from C code
  unsafe extern "C" {
    fn tree_sitter_sam() -> Language;
  }

  fn get_parser() -> Parser {
    let language = unsafe { tree_sitter_sam() };
    let mut parser = Parser::new();
    parser.set_language(&language).unwrap();

    return parser;
  }

  #[test]
  fn test_simple_expression() {
    let source = b"1 + 2;";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    assert!(result.is_ok());
  }

  #[test]
  fn test_variable_assignment() {
    let source = b"
        let x = 5;
        x = x + 1;
    ";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    assert!(result.is_ok());
  }

  #[test]
  fn test_lambda_call() {
    let source = b"
        let f = () => { return 42; };
        let b = f();
    ";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    assert!(result.is_ok());
    assert_eq!(
      result.unwrap().call_stack[0]["b"],
      Value::SamNumber(Number::SamInt(42))
    );
  }

  #[test]
  fn test_nested_return() {
    let source = b"
        let f = () => { if (4 == 4) { return 3 }; };
        let b = f();
    ";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    assert!(result.is_ok());

    assert_eq!(
      result.unwrap().call_stack[0]["b"],
      Value::SamNumber(Number::SamInt(3))
    );
  }

  #[test]
  fn test_nonexistent_var() {
    let source = b"
      let a = b;
    ";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    assert!(!result.is_ok());
  }

  #[test]
  fn test_parameter_handling() {
    let source = b"
      let a = (x, y) => { return x + 5; };
      let b = a(4, 3);
    ";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    assert!(result.is_ok());
    assert_eq!(
      result.unwrap().call_stack[0]["b"],
      Value::SamNumber(Number::SamInt(9))
    );
  }

  #[test]
  fn test_parameter_handling_err() {
    let source = b"
      let a = (x, y) => { return x + 5; };
      let b = a(4);
    ";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    assert!(!result.is_ok());
  }

  #[test]
  fn test_strings() {
    let source = b"
      let a = 'hello';
      let b = 'hello\\nworld';
    ";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    assert!(result.is_ok());

    let result = result.unwrap();

    assert_eq!(
      result.call_stack[0]["a"],
      Value::SamString("hello".to_owned()),
    );

    assert_eq!(
      result.call_stack[0]["b"],
      Value::SamString("hello\nworld".to_owned())
    );
  }

  #[test]
  fn test_string_traits() {
    let source = b"
      let a = 'hello' + ' world';
      let b = 'a' == 'a';
    ";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    assert!(result.is_ok());

    let result = result.unwrap();

    assert_eq!(
      result.call_stack[0]["a"],
      Value::SamString("hello world".to_owned()),
    );
    assert_eq!(
      result.call_stack[0]["b"],
      Value::SamNumber(Number::SamInt(1)),
    );
  }

  #[test]
  fn test_ffi() {
    // create dummy json
    let dir = std::env::temp_dir();
    let path = dir.join("foo.json");
    fs::write(&path, r#"{"bar": "echo 42"}"#).unwrap();

    let source = b"
    interface '/tmp/foo.json' load bar;
    ";

    let mut parser = get_parser();
    let tree = parser.parse(source, None).unwrap();

    let root = tree.root_node();

    let result = evaluate(&root, source, &tree);
    println!("{:#?}", result);
    assert!(result.is_ok());

    let mut result = result.unwrap();

    assert_eq!(
      result.global_scope()["bar"],
      Value::SamForeignFunction(ForeignFunction::new("echo 42".to_owned()))
    );
  }
}

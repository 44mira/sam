#![allow(dead_code)]

use crate::context::Context;
use crate::value::{Number, Value};
use tree_sitter::Node;

fn expect_node(
  node: &Node,
  node_name: &str,
  message: &str,
) -> Result<(), String> {
  if node.kind() != node_name {
    return Err(format!("{} {:#?}", message, node.range()));
  }

  return Ok(());
}

pub fn evaluate(root: &Node, source: &[u8]) -> Result<String, String> {
  expect_node(
    root,
    "source_file",
    "Source file node expected but not found.",
  )?;

  // the variable table/environment, to be passed around as mutable reference
  let mut ctx = Context::new();

  // TODO: handle interface
  let mut walker = root.walk();
  for child in root.named_children(&mut walker) {
    evaluate_statement(child, &mut ctx, source)?;
  }

  println!("{:#?}", ctx);

  return Ok("Evaluation successful".to_owned());
}

fn evaluate_statement(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  // TODO: add other statement types
  match node.kind() {
    "expression_statement" => {
      evaluate_expression(node.child(0).unwrap(), ctx, source)?;
    }
    "variable_declaration" => {
      evaluate_variable_declaration(node, ctx, source)?;
    }
    "assignment" => {
      evaluate_assignment(node, ctx, source)?;
    }
    _ => {
      expect_node(&node, "", "Unknown statement encountered.")?;
    }
  }

  return Ok(());
}

fn evaluate_assignment(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  expect_node(
    &node,
    "assignment",
    "Variable assignment node expected but not found.",
  )?;

  let lhs =
    evaluate_identifier(node.child_by_field_name("lhs").unwrap(), source)?;

  let rhs =
    evaluate_expression(node.child_by_field_name("rhs").unwrap(), ctx, source)?;

  // assign value to existing key
  if !ctx.env.contains_key(&lhs) {
    return Err(format!(
      "Assigning to non-existent variable. {:#?}",
      node.range()
    ));
  }
  ctx.env.entry(lhs).insert_entry(rhs);

  return Ok(());
}

fn evaluate_expression(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  // TODO: add other expression types
  return match node.kind() {
    "literal" => evaluate_literal(node, source),
    "binary_expression" => evaluate_binary_expression(node, ctx, source),
    "identifier" => {
      let varname = evaluate_identifier(node, source)?;

      let Some(value) = ctx.env.get(&varname).cloned() else {
        return Err(format!(
          "Variable {} not defined. {:#?}",
          varname,
          node.range()
        ));
      };

      return Ok(value);
    }
    _ => Err(format!(
      "Unknown expression encountered. {:#?}",
      node.range()
    )),
  };
}

fn evaluate_binary_expression(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  expect_node(
    &node,
    "binary_expression",
    "Binary expression node expected but not found.",
  )?;

  let left = evaluate_expression(
    node.child_by_field_name("left").unwrap(),
    ctx,
    source,
  )?;

  let right = evaluate_expression(
    node.child_by_field_name("right").unwrap(),
    ctx,
    source,
  )?;

  let operator = node.child(1).unwrap().utf8_text(source).unwrap().trim();

  let result = match operator {
    "+" => left + right,
    "*" => left * right,
    "/" => left / right,
    "%" => left % right,
    "-" => left - right,
    "<" => (left < right).into(),
    ">" => (left > right).into(),
    "==" => (left == right).into(),
    "<=" => (left <= right).into(),
    ">=" => (left >= right).into(),
    "!=" => (left != right).into(),
    _ => {
      return Err(format!("Unknown operator encountered. {:#?}", node.range()));
    }
  };

  return Ok(result);
}

fn evaluate_variable_declaration(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  expect_node(
    &node,
    "variable_declaration",
    "Variable declaration not found.",
  )?;

  let mut walker = node.walk();
  for declarator in node.named_children(&mut walker) {
    evaluate_variable_declarator(declarator, ctx, source)?;
  }

  return Ok(());
}

fn evaluate_variable_declarator(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  expect_node(
    &node,
    "variable_declarator",
    "Variable declarator expected but not found.",
  )?;

  // obtain fields
  let ident =
    evaluate_identifier(node.child_by_field_name("variable").unwrap(), source)?;

  // create var in ctx and optionally set value
  ctx.env.insert(ident.to_owned(), Value::Undefined);

  if let Some(value) = node.child_by_field_name("value") {
    let v = evaluate_expression(value, ctx, source)?;
    ctx.env.entry(ident).insert_entry(v);
  }

  return Ok(());
}

fn evaluate_if_expression(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  expect_node(
    &node,
    "if_expression",
    "If expression node expected but not found.",
  )?;

  return Ok(());
}

fn evaluate_statement_block(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<(), String> {
  expect_node(
    &node,
    "statement_block",
    "Statement block node expected but not found.",
  )?;

  return Ok(());
}

fn evaluate_identifier(node: Node, source: &[u8]) -> Result<String, String> {
  expect_node(
    &node,
    "identifier",
    "Identifier node expected but not found.",
  )?;

  // get identifier name
  let ident = node.utf8_text(source).unwrap().to_owned();

  return Ok(ident);
}

fn evaluate_literal(node: Node, source: &[u8]) -> Result<Value, String> {
  expect_node(&node, "literal", "Literal node expected but not found.")?;

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
  expect_node(&node, "number", "Number node expected but not found.")?;

  let value = node.utf8_text(source).unwrap();
  let parsed: Number;

  if value.contains(".") {
    parsed = Number::SamFloat(value.parse().unwrap());
  } else {
    parsed = Number::SamInt(value.parse().unwrap())
  }

  return Ok(parsed);
}

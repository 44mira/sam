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
) -> Result<Value, String> {
  // TODO: add other statement types
  let value = match node.kind() {
    "expression_statement" => {
      evaluate_expression(node.child(0).unwrap(), ctx, source)?
    }
    "variable_declaration" => evaluate_variable_declaration(node, ctx, source)?,
    "assignment" => evaluate_assignment(node, ctx, source)?,
    _ => {
      return Err(format!(
        "Unknown statement encountered. {:#?}",
        node.range()
      ));
    }
  };

  return Ok(value);
}

fn evaluate_assignment(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  expect_node(
    &node,
    "assignment",
    "Variable assignment node expected but not found.",
  )?;

  let lhs =
    evaluate_identifier(node.child_by_field_name("lhs").unwrap(), source)?;

  let rhs =
    evaluate_expression(node.child_by_field_name("rhs").unwrap(), ctx, source)?;

  // assign value to existing key in the call stack
  let Some(var) = ctx.search_in_stack(&lhs) else {
    return Err(format!(
      "Assigning to non-existent variable. {:#?}",
      node.range()
    ));
  };
  *var = rhs;

  return Ok(rhs);
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
    "if_expression" => evaluate_if_expression(node, ctx, source),
    "identifier" => {
      let varname = evaluate_identifier(node, source)?;

      let Some(var) = ctx.search_in_stack(&varname) else {
        return Err(format!(
          "Variable {} not defined. {:#?}",
          varname,
          node.range()
        ));
      };

      // create a new copy for return
      return Ok(var.clone());
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
) -> Result<Value, String> {
  expect_node(
    &node,
    "variable_declaration",
    "Variable declaration not found.",
  )?;

  let mut walker = node.walk();
  for declarator in node.named_children(&mut walker) {
    evaluate_variable_declarator(declarator, ctx, source)?;
  }

  // TODO: Could be set to return the rhs of one declarator instead, but idgaf at this point
  return Ok(Value::Undefined);
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

  // evaluate lhs (if it exists)
  let value = node
    .child_by_field_name("value")
    .map(|n| evaluate_expression(n, ctx, source))
    .transpose()?;

  // assign to current scope
  let scope = ctx.current_scope();
  let entry = scope.entry(ident.to_owned()).or_insert(Value::Undefined);

  if let Some(v) = value {
    *entry = v;
  }

  return Ok(());
}

fn evaluate_if_expression(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  use {Number::SamInt, Value::SamNumber};

  expect_node(
    &node,
    "if_expression",
    "If expression node expected but not found.",
  )?;

  let condition = evaluate_expression(
    node.child_by_field_name("condition").unwrap(),
    ctx,
    source,
  )?;

  let SamNumber(SamInt(cond_result)) = condition else {
    return Err(format!(
      "Expected integer result for condition but not found. {:#?}",
      node.range()
    ));
  };

  let mut value = Value::Undefined;

  if cond_result != 0 {
    value = evaluate_statement_block(
      node.child_by_field_name("consequence").unwrap(),
      ctx,
      source,
    )?;
  } else if let Some(else_arm) = node.child_by_field_name("else") {
    value = match else_arm.kind() {
      "statement_block" => evaluate_expression(else_arm, ctx, source)?,
      "if_expression" => evaluate_if_expression(else_arm, ctx, source)?,
      _ => {
        return Err(format!(
          "Unknown else expression encountered. {:#?}",
          else_arm.range()
        ));
      }
    }
  }

  return Ok(value);
}

fn evaluate_statement_block(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  expect_node(
    &node,
    "statement_block",
    "Statement block node expected but not found.",
  )?;

  ctx.init_scope();

  let mut walker = node.walk();

  // return value of the block
  let mut value = Value::Undefined;

  for statement in node.named_children(&mut walker) {
    if statement.kind() == "return_statement" {
      value = evaluate_return_statement(node, ctx, source)?;

      break;
    }

    evaluate_statement(statement, ctx, source)?;
  }

  println!("{:#?}", ctx);

  ctx.destroy_scope();

  return Ok(value);
}

fn evaluate_return_statement(
  node: Node,
  ctx: &mut Context,
  source: &[u8],
) -> Result<Value, String> {
  expect_node(
    &node,
    "return_statement",
    "Return statement node expected but not found.",
  )?;

  return match node.child_by_field_name("value") {
    Some(v) => evaluate_expression(v, ctx, source),
    None => Ok(Value::Undefined),
  };
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

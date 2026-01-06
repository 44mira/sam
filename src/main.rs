mod context;
mod evaluate;
mod ffi;
mod value;

use evaluate::evaluate;
use tree_sitter::{Language, Parser};

// retrieve Language struct from C code
unsafe extern "C" {
  fn tree_sitter_sam() -> Language;
}

fn main() {
  // set parser language
  let language = unsafe { tree_sitter_sam() };
  let mut parser = Parser::new();
  parser.set_language(&language).unwrap();

  let text = r#"
    interface "/tmp/test.json" load testf;

    let a = testf();
    let b = ls("-la");
  "#;

  let tree = parser.parse(text, None).unwrap();
  let root = &tree.root_node();

  let ctx = evaluate(&root, text.as_bytes(), &tree);

  println!("{:#?}", ctx);
}

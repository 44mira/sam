mod context;
mod evaluate;
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
    let a = 6, b = 4;

    if (4 == 4) {
      let a = 5;
      b = a + b;
    };
    "#;

  let tree = parser.parse(text, None).unwrap();
  let root = tree.root_node();

  match evaluate(&root, text.as_bytes()) {
    Ok(msg) => println!("{msg}"),
    Err(msg) => println!("{msg}"),
  };
}

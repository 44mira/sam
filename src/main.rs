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
        let f = () => { if (4 == 3) {return 42;} else {return 12;}; };
        let b = f();"#;

  let tree = parser.parse(text, None).unwrap();
  let root = &tree.root_node();

  let ctx = evaluate(&root, text.as_bytes(), &tree);

  println!("{:#?}", ctx);
}

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

  let mut input = String::new();

  println!("Input source code:");
  std::io::stdin().read_line(&mut input).unwrap();

  let tree = parser.parse(input, None).unwrap();
  let root = tree.root_node();

  println!("{}", root.to_sexp());
}

fn main() {
  let language = "sam";
  let package = format!("tree-sitter-{}", language);
  let source_dir = format!("{}/src", package);
  let source_file = format!("{}/parser.c", source_dir);

  cc::Build::new()
    .include(&source_dir)
    .file(&source_file)
    .compile(&package);
}

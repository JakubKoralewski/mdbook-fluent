use mdbook::MDBook;
use std::{
    path::PathBuf,
    env::current_dir
};
use mdbook_fluent::FluentPreprocessor;

#[test]
fn the_example_builds_without_errors() {
	let mut root_path = current_dir().unwrap();
    root_path.push(["examples", "example"].iter().collect::<PathBuf>());
    eprintln!("Example book root directory: {:?}", root_path);

    let mut md = MDBook::load(root_path)
        .expect("Unable to load the book");
    md.with_preprocessor(FluentPreprocessor::new());
    md.build().expect("Build failed");
}

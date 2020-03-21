use mdbook::{
	book::Book,
	preprocess::{PreprocessorContext},
	BookItem,
};
pub use mdbook::{
	errors::Error,
	preprocess::{Preprocessor, CmdPreprocessor}
};
use std::{
	process,
	path::{Path, PathBuf},
	io::{self, Read, BufReader},
	fs::File,
	collections::HashMap,
	iter::FromIterator,
	ffi::OsString,
};
use fluent::{
	FluentBundle, FluentResource,
};


pub struct FluentPreprocessor;

const DEFAULT_DIR: &str = "fluent";
const START_EXPR: &str = "{{#fluent";
const START_EXPR_LEN: usize = START_EXPR.len();
const END_EXPR: &str = "}}";
const END_EXPR_LEN: usize = END_EXPR.len();

impl FluentPreprocessor {
	pub fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
		let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
		let calling_ver = semver::Version::parse(&ctx.mdbook_version).unwrap();
		let library_ver = semver::Version::parse(mdbook::MDBOOK_VERSION).unwrap();

		if calling_ver != library_ver {
			eprintln!(
				"Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
				pre.name(),
				mdbook::MDBOOK_VERSION,
				ctx.mdbook_version
			);
		}

		let processed_book = pre.run(&ctx, book)?;
		serde_json::to_writer(io::stdout(), &processed_book)?;

		Ok(())
	}

	pub fn new() -> Self {
		Self
	}

	fn build_fluent_dic(dir: &Path) -> Result<HashMap<OsString, FluentBundle<FluentResource>>, Error> {
		let files: Vec<PathBuf> = Self::visit_dir(dir)?;
		Ok(
			HashMap::from_iter(
				files.into_iter()
				     .filter_map(|file| {
					     if let Some(ext) = file.extension() {
						     if ext != "ftl" {
							     return None;
						     }
					     } else {
						     return None;
					     }
					     let file_name = file.file_stem().unwrap();
					     eprintln!("File: {:#?}", file);
					     let file = File::open(&file).unwrap();
					     let mut reader = BufReader::new(file);
					     let mut contents = String::new();
					     reader.read_to_string(&mut contents).unwrap();
					     let res = FluentResource::try_new(contents)
						     .unwrap_or_else(|(rsrc, e)|
							     panic!("Error interpreting Fluent file ({:?}) contents: {:#?}; {:#?}", &file_name, rsrc, e)
						     );
					     let mut bundle: FluentBundle<FluentResource> = FluentBundle::default();
					     bundle.add_resource(res).expect("Error adding FluentResource to bundle");
					     Some((file_name.into(), bundle))
				     })
			)
		)
	}

	fn visit_dir(dir: &Path) -> Result<Vec<PathBuf>, Error> {
		let mut files: Vec<PathBuf> = Vec::new();
		for file in dir.read_dir()? {
			let file = file?;
			let path = file.path();

			if path.is_dir() {
				files.append(&mut Self::visit_dir(&path)?);
			} else {
				files.push(path);
			}
		}
		Ok(files)
	}
}

impl Preprocessor for FluentPreprocessor {
	fn name(&self) -> &str {
		"fluent"
	}

	fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
		// In testing we want to tell the preprocessor to blow up by setting a
		// particular config value
		let mut dir = DEFAULT_DIR;
		if let Some(nop_cfg) = ctx.config.get_preprocessor(self.name()) {
			if let Some(cfg_dir) = nop_cfg.get("dir") {
				if let Some(cfg_dir) = cfg_dir.as_str() {
					dir = cfg_dir;
				} else {
					eprintln!("Invalid \"dir\" parameter: {:#?}. It should be a string!", cfg_dir);
					process::exit(1);
				}
			}
		}
		let dir = {
			let mut path = ctx.root.clone();
			path.push(dir);
			path
		};


		let dict = Self::build_fluent_dic(&dir)?;
		let mut errors = vec![];
		let end_expr_char = END_EXPR.chars().next().unwrap();
		book.for_each_mut(|item| {
			if let BookItem::Chapter(chapter) = item {
				// eprintln!("{}\n\n", chapter.content);
				let ranges: Vec<(usize, usize)> =
					chapter.content
					       .match_indices(START_EXPR)
					       .map(|(i, _str)| {
						       let mut iter =
							       chapter.content
							              .chars()
							              .enumerate()
							              .skip(i+START_EXPR_LEN)
							              .peekable();
						       let next_i;
						       loop {
							       if let Some((j, char)) = iter.next() {
								       if char == end_expr_char {
									       let (_next_j, next_char) =
										       iter.peek().expect("Unexpected single \"}\" closing char.");
									       if *next_char == end_expr_char {
										       next_i = Some(j);
										       break;
									       }
								       }
							       } else {
								       panic!("No closing #fluent \"}}\".");
							       }
						       }
						       (i, next_i.unwrap())
					       }).collect();
				let mut offset = 0i32;
				for (start, end) in ranges {
					eprintln!("Parsing {:?},{:?}", start, end);
					let start_i = (start as i32 - offset + START_EXPR_LEN as i32) as usize;
					let end_i = (end as i32 - offset) as usize;
					let content: &str
						= &chapter.content[start_i..end_i];
					let content = content.trim();
					let key = content.split('.').collect::<Vec<&str>>();
					if key.is_empty() {
						panic!("Empty fluent tag")
					} else if key.len() == 1 {
						panic!("File not specified")
					} else if key.len() == 2 {
						let file = key[0];
						let tag = key[1];
						if file.is_empty() {
							panic!("Empty file specifier")
						} else if tag.is_empty() {
							panic!("Empty tag specifier")
						}
						let bundle = dict.get(&OsString::from(&file))
						                 .expect("Incorrect file tag");
						let value = bundle
							.get_message(tag)
							.expect("Incorrect fluent tag")
							.value
							.expect("Fluent tag's value does not  exist");
						let resolved = bundle.format_pattern(value, None, &mut errors);
						chapter.content
						       .replace_range(
							       (start as i32 -offset) as usize..(end as i32 + END_EXPR_LEN as i32 - offset) as usize,
							       &resolved
						       );
						// "{{#fluent ch01.Hello-world}}" -> "Hello, world!" = 9({{#fluent) + 1(space) + (16-13)(len) + 2(}})
						let whole_length = (end + END_EXPR_LEN - start) as i32; // cant overflow, logically
						let new_offset = whole_length - resolved.len() as i32;
						offset += new_offset;
					} else if key.len() == 3 {
						let file = key[0];
						let tag = key[1];
						let attribute = key[2];
						if file.is_empty() {
							panic!("Empty file specifier")
						} else if tag.is_empty() {
							panic!("Empty tag specifier")
						} else if attribute.is_empty() {
							panic!("Empty attribute specifier")
						}
						let bundle = dict.get(&OsString::from(&file))
						                 .expect("Incorrect file tag");
						let value = bundle
							.get_message(tag)
							.expect("Incorrect fluent tag");
						let value =
							value
								.attributes
								.get(attribute)
								.expect("Fluent attribute does not exist");
						let resolved = bundle.format_pattern(value, None, &mut errors);
						chapter.content
						       .replace_range(
							       (start as i32 -offset) as usize..(end as i32 + END_EXPR_LEN as i32 - offset) as usize,
							       &resolved
						       );
						let whole_length = (end + END_EXPR_LEN - start) as i32; // cant overflow, logically
						let new_offset = whole_length - resolved.len() as i32;
						offset += new_offset;
					} else {
						panic!("Too many dots")
					}
				}
			}
		});

		Ok(book)
	}

	fn supports_renderer(&self, _renderer: &str) -> bool {
		true
	}
}
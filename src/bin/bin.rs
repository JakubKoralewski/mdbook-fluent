use clap::{App, Arg, ArgMatches, SubCommand};
use std::process;
use mdbook_fluent::{FluentPreprocessor, Preprocessor};

pub fn make_app() -> App<'static, 'static> {
	App::new("mdbook-fluent")
		.about("mdBook preprocessor for variable inteprolation using the Fluent language")
		.subcommand(
			SubCommand::with_name("supports")
				.arg(Arg::with_name("renderer").required(true))
				.about("Check whether a renderer is supported by this preprocessor"),
		)
}

fn main() {
	eprintln!("Running mdbook-fluent");
	let matches = make_app().get_matches();

	let preprocessor = FluentPreprocessor::new();

	if let Some(sub_args) = matches.subcommand_matches("supports") {
		handle_supports(&preprocessor, sub_args);
	} else if let Err(e) = FluentPreprocessor::handle_preprocessing(&preprocessor) {
		eprintln!("{}", e);
		process::exit(1);
	}
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
	let renderer = sub_args.value_of("renderer").expect("Required argument");
	let supported = pre.supports_renderer(&renderer);

	// Signal whether the renderer is supported by exiting with 1 or 0.
	if supported {
		process::exit(0);
	} else {
		process::exit(1);
	}
}

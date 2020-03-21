# mdbook-fluent

A simple WIP preprocessor for [mdBook](https://github.com/rust-lang/mdBook).
It uses the [Fluent language](https://projectfluent.org/) to interpolate
variables inside your book. Building on the shoulders of giants via
the [fluent-rs](https://github.com/projectfluent/fluent-rs) Rust implementation
of the Fluent Project.

**DISCLAIMER:**  
This preprocessor is not designed to be used to translate 
books! This is because [mdBook itself is not designed this way](https://github.com/rust-lang/mdBook/issues/146#issuecomment-354759316)!

## Getting started

Check out the [./examples/example](./examples/example) folder for
a working example.

### Install

```bash
cargo install mdbook-fluent
```

### book.toml

Add this to your `book.toml`:
```toml
[preprocessor.fluent]
```

### Fluent files

In the `fluent` directory create files with the `.ftl`
extension containing your variables. The name of each file
will be used to group the variables into logical chunks. 

You can use the `dir` key in the config to change the name of the directory containing
your `.ftl` files.

Example Fluent File:

```ftl
# fluent/example.ftl
Hello-world = Hello, world!
some-variable =
    .some-attribute = xd
multi-line =
    This is a multi-line Fluent value.
    The spaces before these lines will be automatically
    removed!
```

### Syntax:

In your books use the `{{#fluent FILE_NAME.TAG}}`
or `{{#fluent FILE_NAME.TAG.ATTRIBUTE` syntax, like so:

    # Chapter 1

    ```
    {{#fluent ch01.Hello-world}}
    ```

    ```rust,include
    {{#include ../listing/example.rs}}
    ```

As you can see above, you can put the same `{{#fluent` tags
inside other files that will be included.

### Build

Now simply build your book like you would normally:

```bash
mdbook build
```

## Contributions

I welcome any contributions, whether they would be issues,
feature requests, pull requests or questions.

## License

[MIT](./LICENSE-MIT) and [APACHE](LICENSE-APACHE)
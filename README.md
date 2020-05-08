[![Build Status](https://travis-ci.org/oli-obk/rust-pandoc.svg?branch=master)](https://travis-ci.org/oli-obk/rust-pandoc)

# Instructions

1. [Install pandoc](http://pandoc.org/installing.html)
2. add the pandoc crate to your Cargo.toml

    ```toml
    [dependencies]
    pandoc = "0.8"
   ```

3. create a pandoc builder and execute it

    ```rust
    let mut pandoc = pandoc::new();
    pandoc.add_input("hello_world.md");
    pandoc.set_output(OutputKind::File("hello_world.pdf".to_string()));
    pandoc.execute().unwrap();
    ```

# PDF-output
## Windows specific
install [miktex](http://miktex.org/) or [texlive](https://www.tug.org/texlive/), if your installation paths differ from the default use the `add_latex_path_hint` function to add them to the pandoc builder.

# Common Issues
## file not found errors
use `add_pandoc_path_hint` to add the actual path to pandoc search path. Under windows it can often
be found in `%LOCALAPPDATA%\Pandoc\`, but that path is searched automatically by this crate.

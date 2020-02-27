extern crate pandoc;

use std::path::PathBuf;

#[test]
fn creation() {
    use pandoc::OutputKind;
    use pandoc::PandocOption::*;
    let mut pandoc = pandoc::new();

    pandoc.add_input("cake");
    pandoc.set_output(OutputKind::File(String::from("lie")));
    pandoc.set_chapters();
    pandoc.set_number_sections();
    pandoc.set_latex_template("template.tex");
    pandoc.set_output_format(pandoc::OutputFormat::Beamer, Vec::new());
    pandoc.add_latex_path_hint("D:\\texlive\\2015\\bin\\win32");
    pandoc.set_slide_level(3);
    pandoc.set_toc();
    pandoc.add_option(Strict);
    pandoc.add_option(IndentedCodeClasses("cake".to_string()));
    let path = PathBuf::new();
    pandoc.add_option(Filter(path));
}

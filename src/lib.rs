#![deny(missing_docs)]

//! API that wraps the pandoc command line tool

#[macro_use]
extern crate itertools;

use itertools::Itertools;

/// path to pandoc executable
#[cfg(windows)]
const PANDOC_PATH: &'static [&'static str] = &[
    // this compiles the user's name into the binary, maybe not the greatest idea?
    concat!(env!("LOCALAPPDATA"), r#"\Pandoc\"#),
];
/// path to pandoc executable
#[cfg(not(windows))]
const PANDOC_PATH: &'static [&'static str] = &[
];

/// path where miktex executables can be found
#[cfg(windows)]
const LATEX_PATH: &'static [&'static str] = &[
    r#"C:\Program Files (x86)\MiKTeX 2.9\miktex\bin"#,
    r#"C:\Program Files\MiKTeX 2.9\miktex\bin"#,
];
/// path where miktex executables can be found
#[cfg(not(windows))]
const LATEX_PATH: &'static [&'static str] = &[
    r"/usr/local/bin",
    r"/usr/local/texlive/2015/bin/i386-linux",
];

use std::process::Command;
use std::env;

/// equivalent to the latex document class
#[derive(Debug)]
pub enum DocumentClass {
    /// compact form of report
    Article,
    /// abstract + chapters + custom page for title, abstract and toc
    Report,
    /// no abstract
    Book,
}

pub use DocumentClass::*;

impl DocumentClass {
    fn pandoc_name(&self) -> &'static str {
        match *self {
            Article => "article",
            Report => "report",
            Book => "book",
        }
    }
}

/// the argument builder
#[derive(Debug, Default)]
pub struct Pandoc {
    inputs: Vec<String>,
    output: Option<String>,
    latex_path_hint: Vec<String>,
    pandoc_path_hint: Vec<String>,
    document_class: Option<DocumentClass>,
    bibliography: Option<String>,
    csl: Option<String>,
    toc: bool,
    number_sections: bool,
    template: Option<String>,
    variables: Vec<(String, String)>,
}

use std::convert::Into;
use std::borrow::Cow;

/// does nothing useful, simply gives you a builder object
/// convenience function so you can call pandoc::new()
pub fn new() -> Pandoc { Default::default() }

impl Pandoc {
    /// does nothing useful, simply gives you a builder object
    pub fn new() -> Pandoc { Default::default() }
    /// this path is searched first for latex, then PATH, then some hardcoded hints
    pub fn add_latex_path_hint<'a, T: Into<Cow<'a, str>>>(&mut self, path: T) {
        self.latex_path_hint.push(path.into().into_owned());
    }
    /// this path is searched first for pandoc, then PATH, then some hardcoded hints
    pub fn add_pandoc_path_hint<'a, T: Into<Cow<'a, str>>>(&mut self, path: T) {
        self.pandoc_path_hint.push(path.into().into_owned());
    }

    /// sets or overwrites the document-class
    pub fn set_doc_class(&mut self, class: DocumentClass) {
        self.document_class = Some(class);
    }

    /// adds more input files, the order is relevant
    /// the order of adding the files is the order in which they are processed
    pub fn add_input<'a, T: Into<Cow<'a, str>>>(&mut self, filename: T) {
        self.inputs.push(filename.into().into_owned());
    }
    /// sets or overwrites the output filename
    pub fn set_output<'a, T: Into<Cow<'a, str>>>(&mut self, filename: T) {
        self.output = Some(filename.into().into_owned());
    }

    /// filename of the bibliography database
    pub fn set_bibliography<'a, T: Into<Cow<'a, str>>>(&mut self, filename: T) {
        self.bibliography = Some(filename.into().into_owned());
    }
    /// filename of a citation style file
    pub fn set_csl<'a, T: Into<Cow<'a, str>>>(&mut self, filename: T) {
        self.csl = Some(filename.into().into_owned());
    }
    /// enable table of contents
    pub fn set_toc(&mut self) { self.toc = true; }
    /// prefix section names with indices x.y.z
    pub fn set_number_sections(&mut self) { self.number_sections = true; }
    /// set a custom latex template
    pub fn set_latex_template<'a, T: Into<Cow<'a, str>>>(&mut self, filename: T) {
        self.template = Some(filename.into().into_owned());
    }
    /// set a custom variable
    /// try not to use this, there are convenience functions for most things
    pub fn set_variable<'a, T: Into<Cow<'a, str>>, U: Into<Cow<'a, str>>>(&mut self, key: T, value: U) {
        self.variables.push((key.into().into_owned(), value.into().into_owned()));
    }

    /// actually execute pandoc
    pub fn execute(self) -> Result<(), PandocError> {
        let mut cmd = Command::new("pandoc");
        for input in self.inputs {
            cmd.arg(input);
        }
        cmd.arg("-o").arg(self.output.expect("no output file specified"));
        if let Some(filename) = self.bibliography {
            cmd.arg(format!("--bibliography={}", filename));
        }
        if let Some(filename) = self.csl {
            cmd.arg(format!("--csl={}", filename));
        }
        if self.toc {
            cmd.arg("--toc");
        }
        if self.number_sections {
            cmd.arg("--number-sections");
        }
        if let Some(filename) = self.template {
            cmd.arg(format!("--template={}", filename));
        }
        for (key, val) in self.variables {
            cmd.arg("--variable").arg(format!("{}={}", key, val));
        }
        if let Some(doc_class) = self.document_class {
            cmd.arg("--variable").arg(format!("documentclass={}", doc_class.pandoc_name()));
        }
        let path: String = self.latex_path_hint.iter()
            .chain(self.pandoc_path_hint.iter())
            .map(std::borrow::Borrow::borrow)
            .chain(PANDOC_PATH.into_iter().cloned())
            .chain(LATEX_PATH.into_iter().cloned())
            .chain([env::var("PATH").unwrap()].iter().map(std::borrow::Borrow::borrow))
            .intersperse(";")
            .collect();
        cmd.env("PATH", path);

        match cmd.output() {
            Err(e) => Err(PandocError::IoErr(e)),
            Ok(ref o) if o.status.success() => Ok(()),
            Ok(o) => Err(PandocError::Err(o)),
        }
    }
}

/// Possible errors that can occur before or during pandoc execution
pub enum PandocError {
    /// some kind of IO-Error
    IoErr(std::io::Error),
    /// pandoc execution failed, look at the stderr output
    Err(std::process::Output),
}

impl std::fmt::Debug for PandocError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            PandocError::IoErr(ref e) => write!(fmt, "{:?}", e),
            PandocError::Err(ref e) => {
                try!(write!(fmt, "exit_code: {:?}", e.status.code()));
                try!(write!(fmt, "stdout: {}", String::from_utf8_lossy(&e.stdout)));
                write!(fmt, "stderr: {}", String::from_utf8_lossy(&e.stderr))
            }
        }
    }
}

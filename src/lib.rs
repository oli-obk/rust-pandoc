#![deny(missing_docs)]

//! API that wraps the pandoc command line tool

#[macro_use]
extern crate itertools;

use itertools::Itertools;

use std::io::Write;

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
#[derive(Debug, Clone)]
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

/// typesafe access to -t FORMAT, -w FORMAT, --to=FORMAT, --write=FORMAT
#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// native Haskell
    Native,
    /// JSON version of native AST
    Json,
    /// Plain text
    Plain,
    /// pandoc’s extended markdown
    Markdown,
    /// original unextended markdown
    MarkdownStrict,
    /// PHP Markdown extra extended markdown
    MarkdownPhpextra,
    /// github extended markdown
    MarkdownGithub,
    /// CommonMark markdown
    Commonmark,
    /// reStructuredText
    Rst,
    /// XHTML 1
    Html,
    /// HTML 5
    Html5,
    /// LaTeX
    Latex,
    /// LaTeX beamer slide show
    Beamer,
    /// ConTeXt
    Context,
    /// Groff man
    Man,
    /// MediaWiki markup
    MediaWiki,
    /// DokuWiki markup
    Dokuwiki,
    /// Textile
    Textile,
    /// Emacs Org-Mode
    Org,
    /// GNU Texinfo
    Texinfo,
    /// OPML
    Opml,
    /// DocBook
    Docbook,
    /// Open Document
    OpenDocument,
    /// OpenOffice text document
    Odt,
    /// Word docx
    Docx,
    /// Haddock markup
    Haddock,
    /// Rich text format
    Rtf,
    /// EPUB v2 book
    Epub,
    /// EPUB v3
    Epub3,
    /// FictionBook2 e-book
    Fb2,
    /// AsciiDoc
    Asciidoc,
    /// InDesign ICML
    Icml,
    /// Slidy HTML and javascript slide show
    Slidy,
    /// Slideous HTML and javascript slide show
    Slideous,
    /// DZSlides HTML5 + javascript slide show
    Dzslides,
    /// reveal.js HTML5 + javascript slide show
    Revealjs,
    /// S5 HTML and javascript slide show
    S5,
    /// the path of a custom lua writer (see Custom writers)
    Lua(String),
}

impl OutputFormat {
    fn pandoc_name(&self) -> &'static str {
        use OutputFormat::*;
        match *self {
            Native => "native",
            Json => "json",
            Plain => "plain",
            Markdown => "markdown",
            MarkdownStrict => "markdown_strict",
            MarkdownPhpextra => "markdown_phpextra",
            MarkdownGithub => "markdown_github",
            Commonmark => "commonmark",
            Rst => "rst",
            Html => "html",
            Html5 => "html5",
            Latex => "latex",
            Beamer => "beamer",
            Context => "context",
            Man => "man",
            MediaWiki => "mediawiki",
            Dokuwiki => "dokuwiki",
            Textile => "textile",
            Org => "org",
            Texinfo => "texinfo",
            Opml => "opml",
            Docbook => "docbook",
            OpenDocument => "open_document",
            Odt => "odt",
            Docx => "docx",
            Haddock => "haddock",
            Rtf => "rtf",
            Epub => "epub",
            Epub3 => "epub3",
            Fb2 => "fb2",
            Asciidoc => "asciidoc",
            Icml => "icml",
            Slidy => "slidy",
            Slideous => "slideous",
            Dzslides => "dzslides",
            Revealjs => "revealjs",
            S5 => "s5",
            Lua(_) => unimplemented!(),
        }
    }
}

/// typesafe access to -f FORMAT, -r FORMAT, --from=FORMAT, --read=FORMAT
#[derive(Debug, Clone)]
pub enum InputFormat {
    /// native Haskell
    Native,
    /// JSON version of native AST
    Json,
    /// pandoc’s extended markdown
    Markdown,
    /// original unextended markdown
    MarkdownStrict,
    /// PHP Markdown extra extended markdown
    MarkdownPhpextra,
    /// github extended markdown
    MarkdownGithub,
    /// CommonMark markdown
    Commonmark,
    /// Textile
    Textile,
    /// reStructuredText
    Rst,
    /// HTML
    Html,
    /// DocBook
    DocBook,
    /// txt2tags
    T2t,
    /// Word docx
    Docx,
    /// EPUB
    Epub,
    /// OPML
    Opml,
    /// Emacs Org-Mode
    Org,
    /// MediaWiki markup
    MediaWiki,
    /// TWiki markup
    Twiki,
    /// Haddock markup
    Haddock,
    /// LaTeX
    Latex,
}

impl InputFormat {
    fn pandoc_name(&self) -> &'static str {
        use InputFormat::*;
        match *self {
            Native => "native",
            Json => "json",
            Markdown => "markdown",
            MarkdownStrict => "markdown_strict",
            MarkdownPhpextra => "markdown_phpextra",
            MarkdownGithub => "markdown_github",
            Commonmark => "commonmark",
            Rst => "rst",
            Html => "html",
            Latex => "latex",
            MediaWiki => "mediawiki",
            Textile => "textile",
            Org => "org",
            Opml => "opml",
            Docx => "docx",
            Haddock => "haddock",
            Epub => "epub",
            DocBook => "docbook",
            T2t => "t2t",
            Twiki => "twiki",
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum MarkdownExtensions {
    EscapedLineBreaks,
    BlankBeforeHeader,
    HeaderAttributes,
    AutoIdentifiers,
    ImplicitHeaderReferences,
    BlankBeforeBlockQuote,
    FencedCodeBlocks,
    BacktickCodeBlocks,
    FencedCodeAttributes,
    LineBlocks,
    FancyLists,
    Startnum,
    DefinitionLists,
    ExampleLists,
    TableCaptions,
    SimpleTables,
    MultilineTables,
    GridTables,
    PipeTables,
    PandocTitleBlock,
    YamlMetadataBlock,
    AllSymbolsEscapable,
    IntrawordUnderscores,
    Strikeout,
    Superscript,
    Subscript,
    InlineCodeAttributes,
    TexMathDollars,
    RawHtml,
    MarkdownInHtmlBlocks,
    NativeDivs,
    NativeSpans,
    RawTex,
    LatexMacros,
    ShortcutReferenceLinks,
    ImplicitFigures,
    Footnotes,
    InlineNotes,
    Citations,
    ListsWithoutPrecedingBlankline,
    HardLineBreaks,
    IgnoreLineBreaks,
    TexMathSingleBackslash,
    TexMathDoubleBackslash,
    MarkdownAttribute,
    MmdTitleBlock,
    Abbreviations,
    AutolinkBareUris,
    AsciiIdentifiers,
    LinkAttributes,
    MmdHeaderIdentifiers,
    CompactDefinitionLists,
}

#[derive(Clone, Debug)]
enum InputKind {
    Files(Vec<String>),
    /// passed to the pandoc through stdin
    Pipe(String),
}

#[derive(Clone, Debug)]
enum OutputKind {
    File(String),
    Pipe,
}

/// the argument builder
#[derive(Default, Clone)]
pub struct Pandoc {
    input: Option<InputKind>,
    input_format: Option<InputFormat>,
    output: Option<OutputKind>,
    output_format: Option<OutputFormat>,
    latex_path_hint: Vec<String>,
    pandoc_path_hint: Vec<String>,
    document_class: Option<DocumentClass>,
    bibliography: Option<String>,
    csl: Option<String>,
    toc: bool,
    chapters: bool,
    number_sections: bool,
    template: Option<String>,
    variables: Vec<(String, String)>,
    slide_level: Option<usize>,
    filters: Vec<fn(String) -> String>,
    args: Vec<(String, String)>,
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
    /// sets or overwrites the output format
    pub fn set_output_format(&mut self, format: OutputFormat) {
        self.output_format = Some(format);
    }
    /// sets or overwrites the input format
    pub fn set_input_format(&mut self, format: InputFormat) {
        self.input_format = Some(format);
    }

    /// adds more input files, the order is relevant
    /// the order of adding the files is the order in which they are processed
    pub fn add_input<'a, T: Into<Cow<'a, str>>>(&mut self, filename: T) {
        let filename = filename.into().into_owned();
        match self.input {
            Some(InputKind::Files(ref mut files)) => {
                files.push(filename);
            },
            None => {
                self.input = Some(InputKind::Files(vec![filename]));
            },
            _ => unreachable!(),
        }
    }
    /// sets or overwrites the output filename
    pub fn set_output<'a, T: Into<Cow<'a, str>>>(&mut self, filename: T) {
        self.output = Some(OutputKind::File(filename.into().into_owned()));
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
    /// enable chapters
    pub fn set_chapters(&mut self) { self.chapters = true; }
    /// prefix section names with indices x.y.z
    pub fn set_number_sections(&mut self) { self.number_sections = true; }
    /// set a custom latex template
    pub fn set_latex_template<'a, T: Into<Cow<'a, str>>>(&mut self, filename: T) {
        self.template = Some(filename.into().into_owned());
    }
    /// sets the header level that causes a new slide to be generated
    pub fn set_slide_level(&mut self, level: usize) {
        self.slide_level = Some(level);
    }
    /// set a custom variable
    /// try not to use this, there are convenience functions for most things
    pub fn set_variable<'a, T: Into<Cow<'a, str>>, U: Into<Cow<'a, str>>>(&mut self, key: T, value: U) {
        self.variables.push((key.into().into_owned(), value.into().into_owned()));
    }

    /// closures that take a json string and return a json string
    pub fn add_filter(&mut self, filter: fn(String) -> String) {
        self.filters.push(filter);
    }

    fn run(self) -> Result<Vec<u8>, PandocError> {
        let mut cmd = Command::new("pandoc");
        if let Some(format) = self.output_format {
            cmd.arg("-t").arg(format.pandoc_name());
        }
        if let Some(format) = self.input_format {
            cmd.arg("-f").arg(format.pandoc_name());
        }
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
        for (key, val) in self.args {
            cmd.arg(format!("--{}={}", key, val));
        }
        if let Some(doc_class) = self.document_class {
            cmd.arg("--variable").arg(format!("documentclass={}", doc_class.pandoc_name()));
        }
        if let Some(level) = self.slide_level {
            cmd.arg(format!("--slide-level={}", level));
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
        let output = try!(self.output.ok_or(PandocError::NoOutputSpecified));
        let input = try!(self.input.ok_or(PandocError::NoInputSpecified));
        let input = match input {
            InputKind::Files(files) => {
                for file in files {
                    cmd.arg(file);
                }
                String::new()
            },
            InputKind::Pipe(text) => {
                cmd.stdin(std::process::Stdio::piped());
                text
            },
        };
        match output {
            OutputKind::File(filename) => {
                cmd.arg("-o").arg(filename);
            },
            OutputKind::Pipe => {
                cmd.stdout(std::process::Stdio::piped());
            },
        }
        println!("{:?}", cmd);
        let mut child = try!(cmd.spawn());
        if let Some(ref mut stdin) = child.stdin {
            try!(stdin.write_all(input.as_bytes()));
        }
        let o = try!(child.wait_with_output());
        if o.status.success() {
            Ok(o.stdout)
        } else {
            Err(PandocError::Err(o))
        }
    }

    fn arg<'a, T: Into<Cow<'a, str>>, U: Into<Cow<'a, str>>>(&mut self, key: T, value: U) {
        self.args.push((key.into().into_owned(), value.into().into_owned()));
    }

    /// generate a latex template from the given settings
    /// this function can panic in a lot of places
    pub fn generate_latex_template<'a, T: Into<Cow<'a, str>>>(mut self, filename: T) {
        let format = self.output_format.as_ref().map(OutputFormat::pandoc_name).unwrap();
        self.arg("print-default-template", format);
        let output = self.run().unwrap();
        let filename: &str = &filename.into();
        let mut file = std::fs::File::create(filename).unwrap();
        file.write_all(&output).unwrap();
    }

    /// actually execute pandoc
    pub fn execute(mut self) -> Result<(), PandocError> {
        let filters = std::mem::replace(&mut self.filters, Vec::new());
        if filters.is_empty() {
            let _ = try!(self.run());
            return Ok(());
        }
        let mut pandoc = self.clone();
        self.output = Some(OutputKind::Pipe);
        self.output_format = Some(OutputFormat::Json);
        let o = try!(self.run());
        let o = String::from_utf8(o).unwrap();
        // apply all filters
        let filtered = filters.into_iter().fold(o, |acc, item| item(acc));
        pandoc.input = Some(InputKind::Pipe(filtered));
        pandoc.input_format = Some(InputFormat::Json);
        let _ = try!(pandoc.run());
        Ok(())
    }
}

/// Possible errors that can occur before or during pandoc execution
pub enum PandocError {
    /// some kind of IO-Error
    IoErr(std::io::Error),
    /// pandoc execution failed, look at the stderr output
    Err(std::process::Output),
    /// forgot to specify an output file
    NoOutputSpecified,
    /// forgot to specify any input files
    NoInputSpecified,
}

impl std::convert::From<std::io::Error> for PandocError {
    fn from(val: std::io::Error) -> Self {
        PandocError::IoErr(val)
    }
}

impl std::fmt::Debug for PandocError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            PandocError::IoErr(ref e) => write!(fmt, "{:?}", e),
            PandocError::Err(ref e) => {
                try!(write!(fmt, "exit_code: {:?}", e.status.code()));
                try!(write!(fmt, "stdout: {}", String::from_utf8_lossy(&e.stdout)));
                write!(fmt, "stderr: {}", String::from_utf8_lossy(&e.stderr))
            },
            PandocError::NoOutputSpecified => write!(fmt, "No output file was specified"),
            PandocError::NoInputSpecified => write!(fmt, "No input files were specified"),
        }
    }
}

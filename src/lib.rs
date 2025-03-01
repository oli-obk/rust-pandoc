//! API that wraps the pandoc command line tool

use itertools::Itertools;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str;

/// path where miktex executables can be found
#[cfg(windows)]
const LATEX_PATH: &[&str] = &[
    r#"C:\Program Files (x86)\MiKTeX 2.9\miktex\bin"#,
    r#"C:\Program Files\MiKTeX 2.9\miktex\bin"#,
];
/// path where miktex executables can be found
#[cfg(not(windows))]
const LATEX_PATH: &[&str] = &[r"/usr/local/bin", r"/usr/local/texlive/2015/bin/i386-linux"];

/// character to split path variable on windows
#[cfg(windows)]
const PATH_DELIMIT: &str = ";";

/// character to split path variable on 'other platforms'
#[cfg(not(windows))]
const PATH_DELIMIT: &str = ":";

use std::env;
use std::process::Command;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TrackChanges {
    Accept,
    Reject,
    All,
}

impl std::fmt::Display for TrackChanges {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            TrackChanges::Accept => write!(fmt, "accept"),
            TrackChanges::Reject => write!(fmt, "reject"),
            TrackChanges::All => write!(fmt, "all"),
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EmailObfuscation {
    None,
    Javascript,
    References,
}

impl std::fmt::Display for EmailObfuscation {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            EmailObfuscation::None => write!(fmt, "none"),
            EmailObfuscation::Javascript => write!(fmt, "javascript"),
            EmailObfuscation::References => write!(fmt, "references"),
        }
    }
}

pub type URL = String;

#[derive(PartialEq, Clone, Debug)]
pub enum Tld {
    Chapter,
    Section,
    Part,
}

#[derive(PartialEq, Clone, Debug)]
#[non_exhaustive]
pub enum PandocOption {
    /// --data-dir=DIRECTORY
    DataDir(PathBuf),
    /// --defaults=FILE
    Defaults(PathBuf),
    /// --strict
    Strict,
    /// -R --parse-raw
    ParseRaw,
    /// -S --smart
    Smart,
    /// --old-dashes
    OldDashes,
    /// --base-header-level=NUMBER
    #[deprecated(note = "replaced by ShiftHeadingLevelBy")]
    BaseHeaderLevel(u32),
    /// --shift-heading-level-by=NUMBER,
    ShiftHeadingLevelBy(i32),
    /// --indented-code-classes=STRING
    IndentedCodeClasses(String),
    /// -F PROGRAM --filter=PROGRAM
    Filter(PathBuf),
    /// -L SCRIPT --lua-filter=SCRIPT
    LuaFilter(PathBuf),
    /// --normalize
    Normalize,
    /// -p --preserve-tabs
    PreserveTabs,
    /// --tab-stop=NUMBER
    TabStop(u32),
    /// --track-changes=accept|reject|all
    TrackChanges(TrackChanges),
    /// --extract-media=PATH
    ExtractMedia(PathBuf),
    /// -s --standalone
    Standalone,
    /// --template=FILENAME
    Template(PathBuf),
    /// -M KEY[:VALUE] --metadata=KEY[:VALUE]
    Meta(String, Option<String>),
    /// -V KEY[:VALUE] --variable=KEY[:VALUE]
    Var(String, Option<String>),
    /// -D FORMAT --print-default-template=FORMAT
    PrintDefaultTemplate(String),
    /// --print-default-data-file=FILE
    PrintDefaultDataFile(PathBuf),
    /// --no-wrap
    NoWrap,
    /// --columns=NUMBER
    Columns(u32),
    /// --toc, --table-of-contents
    TableOfContents,
    /// --toc-depth=NUMBER
    TableOfContentsDepth(u32),
    /// --no-highlight
    NoHighlight,
    /// --highlight-style=STYLE
    HighlightStyle(String),
    /// -H FILENAME --include-in-header=FILENAME
    IncludeInHeader(PathBuf),
    /// -B FILENAME --include-before-body=FILENAME
    IncludeBeforeBody(PathBuf),
    /// -A FILENAME --include-after-body=FILENAME
    IncludeAfterBody(PathBuf),
    /// --self-contained
    SelfContained,
    /// --offline
    Offline,
    /// -5 --html5
    Html5,
    /// --html-q-tags
    HtmlQTags,
    /// --ascii
    Ascii,
    /// --reference-links
    ReferenceLinks,
    /// --atx-headers deprecated,
    /// --markdown-headings=atx
    AtxHeaders,
    /// --top-level-division=
    TopLevelDivision(Tld),
    /// -N --number-sections
    NumberSections,
    /// --number-offset=NUMBERS
    NumberOffset(Vec<u32>),
    /// --no-tex-ligatures
    NoTexLigatures,
    /// --listings
    Listings,
    /// -i --incremental
    Incremental,
    /// --slide-level=NUMBER
    SlideLevel(u32),
    /// --section-divs
    SectionDivs,
    /// --default-image-extension=extension
    DefaultImageExtension(String),
    /// --email-obfuscation=none|javascript|references
    EmailObfuscation(EmailObfuscation),
    /// --id-prefix=STRING
    IdPrefix(String),
    /// -T STRING --title-prefix=STRING
    TitlePrefix(String),
    /// -c URL --css=URL
    Css(URL),
    /// --reference-odt=FILENAME
    ReferenceOdt(PathBuf),
    /// --reference-docx=FILENAME
    #[deprecated(note = "replaced by ReferenceDoc")]
    ReferenceDocx(PathBuf),
    /// --reference-doc=FILENAME
    ReferenceDoc(PathBuf),
    /// --epub-stylesheet=FILENAME
    EpubStylesheet(PathBuf),
    /// --epub-cover-image=FILENAME
    EpubCoverImage(PathBuf),
    /// --epub-metadata=FILENAME
    EpubMetadata(PathBuf),
    /// --epub-embed-font=FILE
    EpubEmbedFont(PathBuf),
    /// --epub-chapter-level=NUMBER
    EpubChapterLevel(u32),
    /// --pdf-engine=PROGRAM
    PdfEngine(PathBuf),
    /// --pdf-engine-opt=STRING
    PdfEngineOpt(String),
    /// --citeproc
    Citeproc,
    /// --bibliography=FILE
    Bibliography(PathBuf),
    /// --csl=FILE
    Csl(PathBuf),
    /// --citation-abbreviations=FILE
    CitationAbbreviations(PathBuf),
    /// --natbib
    Natbib,
    /// --biblatex
    Biblatex,
    /// -m[URL] --latexmathml[=URL], --asciimathml[=URL]
    LatexMathML(Option<URL>),
    /// --asciimathml[=URL]
    AsciiMathML(Option<URL>),
    /// --mathml[=URL]
    MathML(Option<URL>),
    /// --mimetex[=URL]
    MimeTex(Option<URL>),
    /// --webtex[=URL]
    WebTex(Option<URL>),
    /// --jsmath[=URL]
    JsMath(Option<URL>),
    /// --mathjax[=URL]
    MathJax(Option<URL>),
    /// --katex[=URL]
    Katex(Option<URL>),
    /// --katex-stylesheet=URL
    KatexStylesheet(URL),
    /// -gladtex
    GladTex,
    /// --trace
    Trace,
    /// --dump-args
    DumpArgs,
    /// --ignore-args
    IgnoreArgs,
    /// --verbose
    Verbose,
    /// --resource-path=PATH
    ResourcePath(Vec<PathBuf>),
    /// +RTS OPTIONS -RTS
    ///
    /// In Pandoc's "A note on security" section of the manual ([link](https://pandoc.org/MANUAL.html#a-note-on-security)), there is a recommendation to set a heap size limit to prevent pathological corner cases.
    ///
    /// The full list of "RTS" options can be found in the Haskell "Runtime control" section of the manual ([link](https://downloads.haskell.org/~ghc/latest/docs/html/users_guide/runtime_control.html)).
    /// The Runtime System options are way more extensive than the -M option, and cover a lot of use-cases that aren't needed while using pandoc in a production setting.
    ///
    /// ## Example Usage
    ///
    /// ```
    /// let mut pandoc = pandoc::new();
    /// pandoc.add_option(pandoc::PandocOption::RuntimeSystem(vec![
    ///   // Limit the heap size to 512 MB while processing an arbitrary input file.
    ///   pandoc::PandocRuntimeSystemOption::MaximumHeapMemory("512M".to_string()),
    /// ]));
    /// ```
    RuntimeSystem(Vec<PandocRuntimeSystemOption>),
    /// --sandbox
    ///
    /// [This option is strongly recommended for any use-case
    /// involving untrusted user
    /// input.](https://pandoc.org/MANUAL.html#option--sandbox) Note
    /// that it does require a pandoc binary compiled with the
    /// `embed_data_files` option, in order to process some formats
    /// such as docx without external file access.
    Sandbox,
    /// Manually specify line endings: crlf (Windows), lf (macOS/Linux/UNIX), or native
    /// (line endings appropriate to the OS on which pandoc is being run). The default is native.
    EOL(String),
}

#[derive(PartialEq, Clone, Debug)]
#[non_exhaustive]
pub enum PandocRuntimeSystemOption {
    /// -M<size>
    MaximumHeapMemory(String),
}

impl PandocOption {
    fn apply<'a>(&self, pandoc: &'a mut Command) -> &'a mut Command {
        use crate::PandocOption::*;
        use crate::Tld::*;
        match *self {
            NumberOffset(ref nums) => {
                let nums = nums.iter().fold(String::new(), |b, n| {
                    if b.is_empty() {
                        format!("{}", n)
                    } else {
                        format!("{}, {}", b, n)
                    }
                });
                pandoc.args([&format!("--number-offset={}", nums)])
            }
            DataDir(ref dir) => pandoc.args([&format!("--data-dir={}", dir.display())]),
            Defaults(ref p) => pandoc.args([&format!("--defaults={}", p.display())]),
            Strict => pandoc.args(["--strict"]),
            ParseRaw => pandoc.args(["--parse-raw"]),
            Smart => pandoc.args(["--smart"]),
            OldDashes => pandoc.args(["--old-dashes"]),
            #[allow(deprecated)]
            BaseHeaderLevel(n) => pandoc.args([&format!("--base-header-level={}", n)]),
            ShiftHeadingLevelBy(n) => pandoc.args([&format!("--shift-heading-level-by={}", n)]),
            IndentedCodeClasses(ref s) => pandoc.args([&format!("--indented-code-classes={}", s)]),
            Filter(ref program) => pandoc.args([&format!("--filter={}", program.display())]),
            LuaFilter(ref script) => pandoc.args([&format!("--lua-filter={}", script.display())]),
            Normalize => pandoc.args(["--normalize"]),
            PreserveTabs => pandoc.args(["--preserve-tabs"]),
            TabStop(n) => pandoc.args([&format!("--tab-stop={}", n)]),
            TrackChanges(ref v) => pandoc.args([&format!("--track-changes={}", v)]),
            ExtractMedia(ref p) => pandoc.args([&format!("--extract-media={}", p.display())]),
            Standalone => pandoc.args(["--standalone"]),
            Template(ref p) => pandoc.args([&format!("--template={}", p.display())]),
            Meta(ref k, Some(ref v)) => pandoc.args(["-M", &format!("{}:{}", k, v)]),
            Meta(ref k, None) => pandoc.args(["-M", k]),
            Var(ref k, Some(ref v)) => pandoc.args(["-V", &format!("{}:{}", k, v)]),
            Var(ref k, None) => pandoc.args(["-V", k]),
            PrintDefaultTemplate(ref f) => {
                pandoc.args([&format!("--print-default-template={}", f)])
            }
            PrintDefaultDataFile(ref f) => {
                pandoc.args([&format!("--print-default-data-file={}", f.display())])
            }
            NoWrap => pandoc.args(["--wrap=none"]),
            Columns(n) => pandoc.args([&format!("--columns={}", n)]),
            TableOfContents => pandoc.args(["--table-of-contents"]),
            TableOfContentsDepth(d) => pandoc.args([&format!("--toc-depth={}", d)]),
            NoHighlight => pandoc.args(["--no-highlight"]),
            HighlightStyle(ref s) => pandoc.args([&format!("--highlight-style={}", s)]),
            IncludeInHeader(ref p) => {
                pandoc.args([&format!("--include-in-header={}", p.display())])
            }
            IncludeBeforeBody(ref p) => {
                pandoc.args([&format!("--include-before-body={}", p.display())])
            }
            IncludeAfterBody(ref p) => {
                pandoc.args([&format!("--include-after-body={}", p.display())])
            }
            SelfContained => pandoc.args(["--self-contained"]),
            Offline => pandoc.args(["--offline"]),
            Html5 => pandoc.args(["--html5"]),
            HtmlQTags => pandoc.args(["--html-q-tags"]),
            Ascii => pandoc.args(["--ascii"]),
            ReferenceLinks => pandoc.args(["--reference-links"]),
            AtxHeaders => pandoc.args(["--markdown-headings=atx"]),
            TopLevelDivision(Chapter) => pandoc.args(["--top-level-division=chapter"]),
            TopLevelDivision(Section) => pandoc.args(["--top-level-division=section"]),
            TopLevelDivision(Part) => pandoc.args(["--top-level-division=part"]),
            NumberSections => pandoc.args(["--number-sections"]),
            NoTexLigatures => pandoc.args(["--no-tex-ligatures"]),
            Listings => pandoc.args(["--listings"]),
            Incremental => pandoc.args(["--incremental"]),
            SlideLevel(n) => pandoc.args(&[format!("--slide-level={}", n)]),
            SectionDivs => pandoc.args(["--section-divs"]),
            DefaultImageExtension(ref s) => {
                pandoc.args(&[format!("--default-image-extension={}", s)])
            }
            EmailObfuscation(o) => pandoc.args(&[format!("--email-obfuscation={}", o)]),
            IdPrefix(ref s) => pandoc.args(&[format!("--id-prefix={}", s)]),
            TitlePrefix(ref s) => pandoc.args(&[format!("--title-prefix={}", s)]),
            Css(ref url) => pandoc.args(&[format!("--css={}", url)]),
            ReferenceOdt(ref file) => pandoc.args(&[format!("--reference-odt={}", file.display())]),
            #[allow(deprecated)]
            ReferenceDocx(ref file) => {
                pandoc.args([&format!("--reference-docx={}", file.display())])
            }
            ReferenceDoc(ref file) => {
                pandoc.args([&format!("--reference-doc={}", file.display())])
            }
            EpubStylesheet(ref file) => {
                pandoc.args([&format!("--epub-stylesheet={}", file.display())])
            }
            EpubCoverImage(ref file) => {
                pandoc.args([&format!("--epub-cover-image={}", file.display())])
            }
            EpubMetadata(ref file) => {
                pandoc.args([&format!("--epub-metadata={}", file.display())])
            }
            EpubEmbedFont(ref file) => {
                pandoc.args([&format!("--epub-embed-font={}", file.display())])
            }
            EpubChapterLevel(num) => pandoc.args([&format!("--epub-chapter-level={}", num)]),
            PdfEngine(ref program) => {
                pandoc.args([&format!("--pdf-engine={}", program.display())])
            }
            PdfEngineOpt(ref s) => pandoc.args([&format!("--pdf-engine-opt={}", s)]),
            Citeproc => pandoc.args(["--citeproc"]),
            Bibliography(ref file) => pandoc.args([&format!("--bibliography={}", file.display())]),
            Csl(ref file) => pandoc.args([&format!("--csl={}", file.display())]),
            CitationAbbreviations(ref f) => {
                pandoc.args([&format!("--citation-abbreviations={}", f.display())])
            }
            Natbib => pandoc.args(["--natbib"]),
            Biblatex => pandoc.args(["--biblatex"]),
            LatexMathML(Some(ref url)) => pandoc.args([&format!("--latexmathml={}", url)]),
            AsciiMathML(Some(ref url)) => pandoc.args([&format!("--asciimathml={}", url)]),
            MathML(Some(ref url)) => pandoc.args([&format!("--mathml={}", url)]),
            MimeTex(Some(ref url)) => pandoc.args([&format!("--mimetex={}", url)]),
            WebTex(Some(ref url)) => pandoc.args([&format!("--webtex={}", url)]),
            JsMath(Some(ref url)) => pandoc.args([&format!("--jsmath={}", url)]),
            MathJax(Some(ref url)) => pandoc.args([&format!("--mathjax={}", url)]),
            Katex(Some(ref url)) => pandoc.args([&format!("--katex={}", url)]),
            LatexMathML(None) => pandoc.args(["--latexmathml"]),
            AsciiMathML(None) => pandoc.args(["--asciimathml"]),
            MathML(None) => pandoc.args(["--mathml"]),
            MimeTex(None) => pandoc.args(["--mimetex"]),
            WebTex(None) => pandoc.args(["--webtex"]),
            JsMath(None) => pandoc.args(["--jsmath"]),
            MathJax(None) => pandoc.args(["--mathjax"]),
            Katex(None) => pandoc.args(["--katex"]),
            KatexStylesheet(ref url) => pandoc.args([&format!("--katex-stylesheet={}", url)]),
            GladTex => pandoc.args(["--gladtex"]),
            Trace => pandoc.args(["--trace"]),
            DumpArgs => pandoc.args(["--dump-args"]),
            IgnoreArgs => pandoc.args(["--ignore-args"]),
            Verbose => pandoc.args(["--verbose"]),
            ResourcePath(ref paths) => {
                let delimiter = if cfg!(windows) { ";" } else { ":" };
                let paths = paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .join(delimiter);
                pandoc.args([&format!("--resource-path={}", paths)])
            }
            RuntimeSystem(ref rts_options) => {
                pandoc.args(["+RTS"]);
                for option in rts_options {
                    match option {
                        PandocRuntimeSystemOption::MaximumHeapMemory(ref s) => {
                            pandoc.args([&format!("-M{}", s)]);
                        }
                    }
                }
                pandoc.args(["-RTS"])
            }
            Sandbox => pandoc.args(["--sandbox"]),
            EOL(ref eol) => pandoc.args([&format!("--eol={}", eol)]),
        }
    }
}

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

pub use crate::DocumentClass::*;

impl std::fmt::Display for DocumentClass {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Article => write!(fmt, "article"),
            Report => write!(fmt, "report"),
            Book => write!(fmt, "book"),
        }
    }
}

/// typesafe access to -t FORMAT, -w FORMAT, --to=FORMAT, --write=FORMAT
#[derive(Debug, Clone)]
#[non_exhaustive]
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
    /// CommonMark markdown with extensions
    CommonmarkX,
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
    /// PDF (via LaTeX)
    Pdf,
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
    /// Other
    Other(String),
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use crate::OutputFormat::*;
        match self {
            Native => write!(fmt, "native"),
            Json => write!(fmt, "json"),
            Plain => write!(fmt, "plain"),
            Markdown => write!(fmt, "markdown"),
            MarkdownStrict => write!(fmt, "markdown_strict"),
            MarkdownPhpextra => write!(fmt, "markdown_phpextra"),
            MarkdownGithub => write!(fmt, "markdown_github"),
            Commonmark => write!(fmt, "commonmark"),
            CommonmarkX => write!(fmt, "commonmark_x"),
            Rst => write!(fmt, "rst"),
            Html => write!(fmt, "html"),
            Html5 => write!(fmt, "html5"),
            Latex => write!(fmt, "latex"),
            Beamer => write!(fmt, "beamer"),
            Context => write!(fmt, "context"),
            Pdf => write!(fmt, "pdf"),
            Man => write!(fmt, "man"),
            MediaWiki => write!(fmt, "mediawiki"),
            Dokuwiki => write!(fmt, "dokuwiki"),
            Textile => write!(fmt, "textile"),
            Org => write!(fmt, "org"),
            Texinfo => write!(fmt, "texinfo"),
            Opml => write!(fmt, "opml"),
            Docbook => write!(fmt, "docbook"),
            OpenDocument => write!(fmt, "open_document"),
            Odt => write!(fmt, "odt"),
            Docx => write!(fmt, "docx"),
            Haddock => write!(fmt, "haddock"),
            Rtf => write!(fmt, "rtf"),
            Epub => write!(fmt, "epub"),
            Epub3 => write!(fmt, "epub3"),
            Fb2 => write!(fmt, "fb2"),
            Asciidoc => write!(fmt, "asciidoc"),
            Icml => write!(fmt, "icml"),
            Slidy => write!(fmt, "slidy"),
            Slideous => write!(fmt, "slideous"),
            Dzslides => write!(fmt, "dzslides"),
            Revealjs => write!(fmt, "revealjs"),
            S5 => write!(fmt, "s5"),
            Lua(_) => unimplemented!(),
            Other(f) => write!(fmt, "{}", f),
        }
    }
}

/// typesafe access to -f FORMAT, -r FORMAT, --from=FORMAT, --read=FORMAT
#[derive(Debug, Clone)]
#[non_exhaustive]
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
    /// CommonMark markdown with extensions
    CommonmarkX,
    /// Textile
    Textile,
    /// reStructuredText
    Rst,
    /// Rich text format \
    /// *Only available as of `pandoc 2.14.2 (2021-08-21)`*
    Rtf,
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
    /// Other
    Other(String),
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use crate::InputFormat::*;
        match self {
            Native => write!(fmt, "native"),
            Json => write!(fmt, "json"),
            Markdown => write!(fmt, "markdown"),
            MarkdownStrict => write!(fmt, "markdown_strict"),
            MarkdownPhpextra => write!(fmt, "markdown_phpextra"),
            MarkdownGithub => write!(fmt, "markdown_github"),
            Commonmark => write!(fmt, "commonmark"),
            CommonmarkX => write!(fmt, "commonmark_x"),
            Rst => write!(fmt, "rst"),
            Rtf => write!(fmt, "rtf"),
            Html => write!(fmt, "html"),
            Latex => write!(fmt, "latex"),
            MediaWiki => write!(fmt, "mediawiki"),
            Textile => write!(fmt, "textile"),
            Org => write!(fmt, "org"),
            Opml => write!(fmt, "opml"),
            Docx => write!(fmt, "docx"),
            Haddock => write!(fmt, "haddock"),
            Epub => write!(fmt, "epub"),
            DocBook => write!(fmt, "docbook"),
            T2t => write!(fmt, "t2t"),
            Twiki => write!(fmt, "twiki"),
            Other(f) => write!(fmt, "{}", f),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MarkdownExtension {
    Smart,
    Attributes,
    EscapedLineBreaks,
    BlankBeforeHeader,
    HeaderAttributes,
    AutoIdentifiers,
    ImplicitHeaderReferences,
    BlankBeforeBlockQuote,
    FencedDivs,
    FencedCodeBlocks,
    BacktickCodeBlocks,
    FencedCodeAttributes,
    LineBlocks,
    FancyLists,
    Startnum,
    TaskLists,
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
    RawAttribute,
    RawHtml,
    MarkdownInHtmlBlocks,
    NativeDivs,
    NativeSpans,
    BracketedSpans,
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
    RebaseRelativePaths,
    Other(String),
}

impl std::fmt::Display for MarkdownExtension {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use crate::MarkdownExtension::*;
        match self {
            Smart => write!(fmt, "smart"),
            Attributes => write!(fmt, "attributes"),
            EscapedLineBreaks => write!(fmt, "escaped_line_breaks"),
            BlankBeforeHeader => write!(fmt, "blank_before_header"),
            HeaderAttributes => write!(fmt, "header_attributes"),
            AutoIdentifiers => write!(fmt, "auto_identifiers"),
            ImplicitHeaderReferences => write!(fmt, "implicit_header_references"),
            BlankBeforeBlockQuote => write!(fmt, "blank_before_block_quote"),
            FencedDivs => write!(fmt, "fenced_divs"),
            FencedCodeBlocks => write!(fmt, "fenced_code_blocks"),
            BacktickCodeBlocks => write!(fmt, "backtick_code_blocks"),
            FencedCodeAttributes => write!(fmt, "fenced_code_attributes"),
            LineBlocks => write!(fmt, "line_blocks"),
            FancyLists => write!(fmt, "fancy_lists"),
            Startnum => write!(fmt, "startnum"),
            TaskLists => write!(fmt, "task_lists"),
            DefinitionLists => write!(fmt, "definition_lists"),
            ExampleLists => write!(fmt, "example_lists"),
            TableCaptions => write!(fmt, "table_captions"),
            SimpleTables => write!(fmt, "simple_tables"),
            MultilineTables => write!(fmt, "multiline_tables"),
            GridTables => write!(fmt, "grid_tables"),
            PipeTables => write!(fmt, "pipe_tables"),
            PandocTitleBlock => write!(fmt, "pandoc_title_block"),
            YamlMetadataBlock => write!(fmt, "yaml_metadata_block"),
            AllSymbolsEscapable => write!(fmt, "all_symbols_escapable"),
            IntrawordUnderscores => write!(fmt, "intraword_underscores"),
            Strikeout => write!(fmt, "strikeout"),
            Superscript => write!(fmt, "superscript"),
            Subscript => write!(fmt, "subscript"),
            InlineCodeAttributes => write!(fmt, "inline_code_attributes"),
            TexMathDollars => write!(fmt, "tex_math_dollars"),
            RawAttribute => write!(fmt, "raw_attribute"),
            RawHtml => write!(fmt, "raw_html"),
            MarkdownInHtmlBlocks => write!(fmt, "markdown_in_html_blocks"),
            NativeDivs => write!(fmt, "native_divs"),
            NativeSpans => write!(fmt, "native_spans"),
            BracketedSpans => write!(fmt, "bracketed_spans"),
            RawTex => write!(fmt, "raw_tex"),
            LatexMacros => write!(fmt, "latex_macros"),
            ShortcutReferenceLinks => write!(fmt, "shortcut_reference_links"),
            ImplicitFigures => write!(fmt, "implicit_figures"),
            Footnotes => write!(fmt, "footnotes"),
            InlineNotes => write!(fmt, "inline_notes"),
            Citations => write!(fmt, "citations"),
            ListsWithoutPrecedingBlankline => write!(fmt, "lists_without_preceding_blankline"),
            HardLineBreaks => write!(fmt, "hard_line_breaks"),
            IgnoreLineBreaks => write!(fmt, "ignore_line_breaks"),
            TexMathSingleBackslash => write!(fmt, "tex_math_single_backslash"),
            TexMathDoubleBackslash => write!(fmt, "tex_math_double_backslash"),
            MarkdownAttribute => write!(fmt, "markdown_attribute"),
            MmdTitleBlock => write!(fmt, "Mmd_title_block"),
            Abbreviations => write!(fmt, "abbreviations"),
            AutolinkBareUris => write!(fmt, "autolink_bare_uris"),
            AsciiIdentifiers => write!(fmt, "ascii_identifiers"),
            LinkAttributes => write!(fmt, "link_attributes"),
            MmdHeaderIdentifiers => write!(fmt, "mmd_header_identifiers"),
            CompactDefinitionLists => write!(fmt, "compact_definition_lists"),
            RebaseRelativePaths => write!(fmt, "rebase_relative_paths"),
            Other(e) => write!(fmt, "{}", e),
        }
    }
}

#[derive(Clone, Debug)]
pub enum InputKind {
    Files(Vec<PathBuf>),
    /// passed to the pandoc executable through stdin
    Pipe(String),
}

/// Specify whether to generate a file or pipe the output to stdout.
#[derive(Clone, Debug)]
pub enum OutputKind {
    File(PathBuf),
    Pipe,
}

/// the argument builder
#[derive(Default, Clone)]
pub struct Pandoc {
    input: Option<InputKind>,
    input_format: Option<(InputFormat, Vec<MarkdownExtension>)>,
    output: Option<OutputKind>,
    output_format: Option<(OutputFormat, Vec<MarkdownExtension>)>,
    latex_path_hint: Vec<PathBuf>,
    pandoc_path_hint: Vec<PathBuf>,
    filters: Vec<Rc<dyn Fn(String) -> String>>,
    args: Vec<(String, String)>,
    options: Vec<PandocOption>,
    print_pandoc_cmdline: bool,
}

/// Convenience function to call Pandoc::new()
pub fn new() -> Pandoc {
    Pandoc::new()
}

impl Pandoc {
    /// Get a new Pandoc object
    /// This function returns a builder object to configure the Pandoc
    /// execution.
    pub fn new() -> Pandoc {
        Pandoc {
            print_pandoc_cmdline: false,
            ..Default::default()
        }
    }

    /// Add a path hint to search for the LaTeX executable.
    ///
    /// The supplied path is searched first for the latex executable, then the environment variable
    /// `PATH`, then some hard-coded location hints.
    pub fn add_latex_path_hint<T: AsRef<Path> + ?Sized>(&mut self, path: &T) -> &mut Pandoc {
        self.latex_path_hint.push(path.as_ref().to_owned());
        self
    }

    /// Add a path hint to search for the Pandoc executable.
    ///
    /// The supplied path is searched first for the Pandoc executable, then the environment variable `PATH`, then
    /// some hard-coded location hints.
    pub fn add_pandoc_path_hint<T: AsRef<Path> + ?Sized>(&mut self, path: &T) -> &mut Pandoc {
        self.pandoc_path_hint.push(path.as_ref().to_owned());
        self
    }

    /// Set or overwrite the document-class.
    pub fn set_doc_class(&mut self, class: DocumentClass) -> &mut Pandoc {
        self.options.push(PandocOption::Var(
            "documentclass".to_string(),
            Some(class.to_string()),
        ));
        self
    }

    /// Set whether Pandoc should print the used command-line
    ///
    /// If set to true, the command-line to execute pandoc (as a subprocess)
    /// will be displayed on stdout.
    pub fn set_show_cmdline(&mut self, flag: bool) -> &mut Pandoc {
        self.print_pandoc_cmdline = flag;
        self
    }

    /// Set or overwrite the output format.
    pub fn set_output_format(
        &mut self,
        format: OutputFormat,
        extensions: Vec<MarkdownExtension>,
    ) -> &mut Pandoc {
        self.output_format = Some((format, extensions));
        self
    }
    /// Set or overwrite the input format
    pub fn set_input_format(
        &mut self,
        format: InputFormat,
        extensions: Vec<MarkdownExtension>,
    ) -> &mut Pandoc {
        self.input_format = Some((format, extensions));
        self
    }

    /// Add additional input files
    ///
    /// The order of adding the files is the order in which they are processed, hence the order is
    /// important.
    /// This function does not work, if input has been already set to standard input using
    /// [`set_input`](#method.set_input_format).
    pub fn add_input<T: AsRef<Path> + ?Sized>(&mut self, filename: &T) -> &mut Pandoc {
        let filename = filename.as_ref().to_owned();
        match self.input {
            Some(InputKind::Files(ref mut files)) => {
                files.push(filename);
            }
            Some(InputKind::Pipe(_)) => panic!(
                "Input has been set to stdin already, \
                                            adding input file names is impossible"
            ),
            None => {
                self.input = Some(InputKind::Files(vec![filename]));
            }
        };
        self
    }

    /// Set input for Pandoc.
    ///
    /// The input is given with `pandoc::InputKind` and overrides any inputs already
    /// supplied.
    ///
    /// # Example
    ///
    /// ```
    /// // pass in a string using standard input:
    /// let markdown = "**very** _important".into();
    /// let mut p = pandoc::new(); // assign to variable to increase life time
    /// p.set_input(pandoc::InputKind::Pipe(markdown));
    pub fn set_input(&mut self, input: InputKind) -> &mut Pandoc {
        self.input = Some(input);
        self
    }

    /// Set or overwrite the output filename.
    pub fn set_output(&mut self, output: OutputKind) -> &mut Pandoc {
        self.output = Some(output);
        self
    }

    /// Set the file name of the bibliography database.
    pub fn set_bibliography<T: AsRef<Path> + ?Sized>(&mut self, filename: &T) -> &mut Pandoc {
        self.options
            .push(PandocOption::Bibliography(filename.as_ref().to_owned()));
        self
    }

    /// Set the filename of the citation style file.
    pub fn set_csl<T: AsRef<Path> + ?Sized>(&mut self, filename: &T) -> &mut Pandoc {
        self.options
            .push(PandocOption::Csl(filename.as_ref().to_owned()));
        self
    }

    /// Enable the generation of a table of contents
    ///
    /// By default, documents are transformed as they are. If this option is set, a table of
    /// contents is added right in front of the actual document.
    pub fn set_toc(&mut self) -> &mut Pandoc {
        self.options.push(PandocOption::TableOfContents);
        self
    }

    /// Treat top-level headers as chapters in LaTeX, ConTeXt, and DocBook output.
    pub fn set_chapters(&mut self) -> &mut Pandoc {
        self.options
            .push(PandocOption::TopLevelDivision(Tld::Chapter));
        self
    }

    /// Set custom prefix for sections.
    ///
    /// If this function is called, all sections will be numbered. Normally, sections in LaTeX,
    /// ConTeXt, HTML, or EPUB output are unnumbered.
    pub fn set_number_sections(&mut self) -> &mut Pandoc {
        self.options.push(PandocOption::NumberSections);
        self
    }

    /// Set a custom latex template.
    pub fn set_latex_template<T: AsRef<Path> + ?Sized>(&mut self, filename: &T) -> &mut Pandoc {
        self.options
            .push(PandocOption::Template(filename.as_ref().to_owned()));
        self
    }

    /// Set the header level that causes a new slide to be generated.
    pub fn set_slide_level(&mut self, level: u32) -> &mut Pandoc {
        self.options.push(PandocOption::SlideLevel(level));
        self
    }

    /// Set a custom variable.
    ///
    /// This method sets a custom Pandoc variable. It is adviced not to use this function, because
    /// there are convenience functions for most of the available variables.
    pub fn set_variable<T: AsRef<str> + ?Sized, U: AsRef<str> + ?Sized>(
        &mut self,
        key: &T,
        value: &U,
    ) -> &mut Pandoc {
        self.options.push(PandocOption::Var(
            key.as_ref().to_owned(),
            Some(value.as_ref().to_owned()),
        ));
        self
    }

    /// Add a Pandoc filter.
    ///
    /// Pandoc parses any of the supported input formats to an abstract syntax tree (AST). If a
    /// filter is specified, it will receive a JSON representation of this AST and can transform it
    /// to its liking and add/modify/remove elements. The output is then passed back to Pandoc.
    ///
    /// The provided filter function must live at least as long as the Pandoc instance,
    /// which will typically be achieved by making it a function, or else a closure which
    /// does not attempt to hold references to anything which isn't `'static`.
    pub fn add_filter<F>(&mut self, filter: F) -> &mut Pandoc
    where
        F: 'static + Fn(String) -> String,
    {
        self.filters.push(Rc::new(filter));
        self
    }

    /// Add a [PandocOption](PandocOption.t.html).
    pub fn add_option(&mut self, option: PandocOption) -> &mut Pandoc {
        self.options.push(option);
        self
    }

    pub fn add_options(&mut self, options: &[PandocOption]) -> &mut Pandoc {
        self.options.extend_from_slice(options);
        self
    }

    fn run(self) -> Result<Vec<u8>, PandocError> {
        let mut cmd = Command::new("pandoc");
        if let Some((ref format, ref extensions)) = self.input_format {
            use std::fmt::Write;
            let mut arg = format.to_string();
            for extension in extensions {
                write!(arg, "+{}", extension).unwrap();
            }
            cmd.arg("-f").arg(arg);
        }
        for (key, val) in self.args {
            cmd.arg(format!("--{}={}", key, val));
        }

        #[cfg(windows)]
        let os_specific_paths: &[PathBuf] = &[
            PathBuf::from(env::var("LOCALAPPDATA").expect("LOCALAPPDATA not set")).join(r#"\Pandoc\"#)
        ];
        #[cfg(not(windows))]
        let os_specific_paths: &[PathBuf] = &[];

        let path: String = Itertools::intersperse(
            self.latex_path_hint
                .iter()
                .chain(self.pandoc_path_hint.iter())
                .chain(os_specific_paths)
                .map(|p| p.to_str().expect("non-utf8 path"))
                .chain(LATEX_PATH.iter().cloned())
                .chain(
                    [env::var("PATH").unwrap()]
                        .iter()
                        .map(std::borrow::Borrow::borrow),
                ),
            PATH_DELIMIT,
        )
        .collect();
        cmd.env("PATH", path);
        let output = self.output.ok_or(PandocError::NoOutputSpecified)?;
        let input = self.input.ok_or(PandocError::NoInputSpecified)?;
        let input = match input {
            InputKind::Files(files) => {
                for file in files {
                    cmd.arg(file);
                }
                String::new()
            }
            InputKind::Pipe(text) => {
                cmd.stdin(std::process::Stdio::piped());
                text
            }
        };
        match output {
            OutputKind::File(filename) => {
                cmd.arg("-o").arg(filename);
            }
            OutputKind::Pipe => {
                match self.output_format {
                    Some((OutputFormat::Pdf, ..)) => {
                        cmd.arg("-o").arg("-").stdout(std::process::Stdio::piped())
                    }
                    _ => cmd.stdout(std::process::Stdio::piped()),
                };
            }
        }

        // always capture stderr
        cmd.stderr(std::process::Stdio::piped());

        if let Some((ref format, ref extensions)) = self.output_format {
            use std::fmt::Write;
            let mut arg = format.to_string();
            for extension in extensions {
                write!(arg, "+{}", extension).unwrap();
            }
            cmd.arg("-t").arg(arg);
        }

        for opt in self.options {
            opt.apply(&mut cmd);
        }
        if self.print_pandoc_cmdline {
            println!("{:?}", cmd);
        }
        let mut child = cmd.spawn()?;
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(input.as_bytes())?;
        }
        let o = child.wait_with_output()?;
        if o.status.success() {
            Ok(o.stdout)
        } else {
            Err(PandocError::Err(o))
        }
    }

    /// Add a raw command-line argument. You should generally use one of the
    /// convenience functions instead.
    pub fn arg<T: AsRef<str> + ?Sized, U: AsRef<str> + ?Sized>(
        &mut self,
        key: &T,
        value: &U,
    ) -> &mut Pandoc {
        self.args
            .push((key.as_ref().to_owned(), value.as_ref().to_owned()));
        self
    }

    /// generate a latex template from the given settings
    ///
    /// Warning: this function can panic in a lot of places.
    pub fn generate_latex_template<T: AsRef<str> + ?Sized>(mut self, filename: &T) {
        let mut format = None;
        if let Some((ref f, ref ext)) = self.output_format {
            let mut s = f.to_string();
            for ext in ext {
                use std::fmt::Write;
                write!(&mut s, "+{}", ext).unwrap();
            }
            format = Some(s);
        }
        let format = format.unwrap();
        self.arg("print-default-template", &format);
        let output = self.run().unwrap();
        let mut file = std::fs::File::create(filename.as_ref()).unwrap();
        file.write_all(&output).unwrap();
    }

    fn preprocess(&mut self) -> Result<(), PandocError> {
        let filters = std::mem::take(&mut self.filters);

        if filters.is_empty() {
            return Ok(());
        }

        let mut pre = new();
        pre.pandoc_path_hint = self.pandoc_path_hint.clone();
        pre.latex_path_hint = self.latex_path_hint.clone();
        pre.output = Some(OutputKind::Pipe);
        pre.set_output_format(OutputFormat::Json, Vec::new());
        pre.input = self.input.take();
        pre.print_pandoc_cmdline = self.print_pandoc_cmdline;
        match self.input_format.take() {
            None => self.input_format = Some((InputFormat::Json, Vec::new())),
            Some((fmt, ext)) => {
                pre.input_format = Some((fmt, ext));
                self.input_format = Some((InputFormat::Json, Vec::new()));
            }
        }
        let o = pre.run()?;
        let o = String::from_utf8(o).unwrap();
        // apply all filters
        let filtered = filters.into_iter().fold(o, |acc, item| item(acc));
        self.input = Some(InputKind::Pipe(filtered));
        Ok(())
    }

    /// Execute the Pandoc configured command.
    ///
    /// A successful Pandoc run can return either the path to a file written by
    /// the operation, or the result of the operation from `stdio`.
    ///
    /// The `PandocOutput` variant returned depends on the `OutputKind`
    /// configured:
    pub fn execute(mut self) -> Result<PandocOutput, PandocError> {
        self.preprocess()?;
        let output_format = self.output_format.clone();
        let output_kind = self.output.clone();
        let output = self.run()?;

        match output_kind {
            Some(OutputKind::File(name)) => Ok(PandocOutput::ToFile(name)),
            Some(OutputKind::Pipe) => match output_format {
                Some((OutputFormat::Pdf | OutputFormat::Docx, ..)) => Ok(PandocOutput::ToBufferRaw(output)),

                _ => match String::from_utf8(output) {
                    Ok(string) => Ok(PandocOutput::ToBuffer(string)),
                    Err(err) => Err(PandocError::from(err.utf8_error())),
                },
            },
            None => Err(PandocError::NoOutputSpecified),
        }
    }
}

/// The output from Pandoc: the file written to, or a buffer with its output.
pub enum PandocOutput {
    /// The results of the pandoc operation are stored in `Path`
    ToFile(PathBuf),
    /// The results of the pandoc operation are returned as a `String` (constructed from the UTF-8
    /// stream returned by pandoc). This will be the case for text-based formats.
    ToBuffer(String),
    /// The results of the pandoc operation are returned as a `Vec<u8>`. This will be the case for
    /// binary formats such as PDF.
    ToBufferRaw(Vec<u8>),
}

/// Possible errors that can occur before or during pandoc execution
pub enum PandocError {
    /// conversion from UTF-8 failed; includes valid-up-to byte count.
    BadUtf8Conversion(usize),
    /// some kind of IO-Error
    IoErr(std::io::Error),
    /// pandoc execution failed, provide output from stderr
    Err(std::process::Output),
    /// forgot to specify an output file
    NoOutputSpecified,
    /// forgot to specify any input files
    NoInputSpecified,
    /// pandoc executable not found
    PandocNotFound,
}

impl std::convert::From<std::io::Error> for PandocError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => PandocError::PandocNotFound,
            _ => PandocError::IoErr(err),
        }
    }
}

impl std::convert::From<std::str::Utf8Error> for PandocError {
    fn from(error: std::str::Utf8Error) -> Self {
        PandocError::BadUtf8Conversion(error.valid_up_to())
    }
}

impl std::fmt::Debug for PandocError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            PandocError::IoErr(ref e) => write!(fmt, "{:?}", e),
            PandocError::Err(ref e) => {
                write!(fmt, "exit_code: {:?}", e.status.code())?;
                write!(fmt, "stdout: {}", String::from_utf8_lossy(&e.stdout))?;
                write!(fmt, "stderr: {}", String::from_utf8_lossy(&e.stderr))
            }
            PandocError::NoOutputSpecified => write!(fmt, "No output file was specified"),
            PandocError::NoInputSpecified => write!(fmt, "No input files were specified"),
            PandocError::PandocNotFound => {
                write!(fmt, "Pandoc not found, did you forget to install pandoc?")
            }
            PandocError::BadUtf8Conversion(byte) => write!(
                fmt,
                "UTF-8 conversion of pandoc output failed after byte {}.",
                byte
            ),
        }
    }
}

impl std::fmt::Display for PandocError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        std::fmt::Debug::fmt(self, fmt)
    }
}

impl std::error::Error for PandocError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            PandocError::IoErr(ref e) => Some(e),
            _ => None,
        }
    }
}

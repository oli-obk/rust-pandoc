//! API that wraps the pandoc command line tool

#[macro_use]
extern crate itertools;

use itertools::Itertools;

use std::io::Write;
use std::path::Path;

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

#[derive(Clone, Debug)]
/// allow to choose an output format with or without extensions
pub enum OutputFormatExt {
    /// a predefined pandoc format
    Fmt(OutputFormat),
    /// allows formats like markdown+pipetables+gridtables
    FmtExt(OutputFormat, Vec<MarkdownExtension>),
}

impl OutputFormatExt {
    fn render(&self) -> String {
        match *self {
            OutputFormatExt::Fmt(ref s) => s.to_string(),
            OutputFormatExt::FmtExt(ref s, ref ext) => {
                let mut s = s.to_string();
                for e in ext {
                    s.push_str("+");
                    s.push_str(&e.to_string());
                }
                s
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TrackChanges { Accept, Reject, All }

impl std::fmt::Display for TrackChanges {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            TrackChanges::Accept => write!(fmt, "accept"),
            TrackChanges::Reject => write!(fmt, "reject"),
            TrackChanges::All    => write!(fmt, "all"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EmailObfuscation { None, Javascript, References }

impl std::fmt::Display for EmailObfuscation {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            EmailObfuscation::None => write!(fmt, "none"),
            EmailObfuscation::Javascript => write!(fmt, "javascript"),
            EmailObfuscation::References => write!(fmt, "references"),
        }
    }
}

pub type URL = str;

#[derive(Clone, Debug)]
pub enum PandocOption<'a> {
    /// -t FORMAT  --to=FORMAT
    To(OutputFormatExt),
    /// --data-dir=DIRECTORY
    DataDir(&'a Path),
    /// --strict
    Strict,
    /// -R --parse-raw
    ParseRaw,
    /// -S --smart
    Smart,
    /// --old-dashes
    OldDashes,
    /// --base-header-level=NUMBER
    BaseHeaderLevel(u32),
    /// --indented-code-classes=STRING
    IndentedCodeClasses(&'a str),
    /// -F PROGRAM --filter=PROGRAM
    Filter(&'a Path),
    /// --normalize
    Normalize,
    /// -p --preserve-tabs
    PreserveTabs,
    /// --tab-stop=NUMBER
    TabStop(u32),
    /// --track-changes=accept|reject|all
    TrackChanges(TrackChanges),
    /// --extract-media=PATH
    ExtractMedia(&'a Path),
    /// -s --standalone
    Standalone,
    /// --template=FILENAME
    Template(&'a Path),
    /// -M KEY[:VALUE] --metadata=KEY[:VALUE]
    Meta(&'a str, Option<&'a str>),
    /// -V KEY[:VALUE] --variable=KEY[:VALUE]
    Var(&'a str, Option<&'a str>),
    /// -D FORMAT --print-default-template=FORMAT
    PrintDefaultTemplate(&'a str),
    /// --print-default-data-file=FILE
    PrintDefaultDataFile(&'a Path),
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
    HighlightStyle(&'a str),
    /// -H FILENAME --include-in-header=FILENAME
    IncludeInHeader(&'a Path),
    /// -B FILENAME --include-before-body=FILENAME
    IncludeBeforeBody(&'a Path),
    /// -A FILENAME --include-after-body=FILENAME
    IncludeAfterBody(&'a Path),
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
    /// --atx-headers
    AtxHeaders,
    /// --chapters
    Chapters,
    /// -N --number-sections
    NumberSections,
    /// --number-offset=NUMBERS
    NumberOffset(&'a [u32]),
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
    DefaultImageExtension(&'a str),
    /// --email-obfuscation=none|javascript|references
    EmailObfuscation(EmailObfuscation),
    /// --id-prefix=STRING
    IdPrefix(&'a str),
    /// -T STRING --title-prefix=STRING
    TitlePrefix(&'a str),
    /// -c URL --css=URL
    Css(&'a URL),
    /// --reference-odt=FILENAME
    ReferenceOdt(&'a Path),
    /// --reference-docx=FILENAME
    ReferenceDocx(&'a Path),
    /// --epub-stylesheet=FILENAME
    EpubStylesheet(&'a Path),
    /// --epub-cover-image=FILENAME
    EpubCoverImage(&'a Path),
    /// --epub-metadata=FILENAME
    EpubMetadata(&'a Path),
    /// --epub-embed-font=FILE
    EpubEmbedFont(&'a Path),
    /// --epub-chapter-level=NUMBER
    EpubChapterLevel(u32),
    /// --latex-engine=PROGRAM
    LatexEngine(&'a Path),
    /// --latex-engine-opt=STRING
    LatexEngineOpt(&'a str),
    /// --bibliography=FILE
    Bibliography(&'a Path),
    /// --csl=FILE
    Csl(&'a Path),
    /// --citation-abbreviations=FILE
    CitationAbbreviations(&'a Path),
    /// --natbib
    Natbib,
    /// --biblatex
    Biblatex,
    /// -m[URL] --latexmathml[=URL], --asciimathml[=URL]
    LatexMathML(Option<&'a URL>),
    /// --asciimathml[=URL]
    AsciiMathML(Option<&'a URL>),
    /// --mathml[=URL]
    MathML(Option<&'a URL>),
    /// --mimetex[=URL]
    MimeTex(Option<&'a URL>),
    /// --webtex[=URL]
    WebTex(Option<&'a URL>),
    /// --jsmath[=URL]
    JsMath(Option<&'a URL>),
    /// --mathjax[=URL]
    MathJax(Option<&'a URL>),
    /// --katex[=URL]
    Katex(Option<&'a URL>),
    /// --katex-stylesheet=URL
    KatexStylesheet(&'a URL),
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
}

impl<'a> PandocOption<'a> {
    fn apply<'c>(&self, pandoc: &'c mut Command) -> &'c mut Command {
        use PandocOption::*;
        match *self {

            NumberOffset(nums)       => {
                let nums = nums.iter()
                    .fold(String::new(),
                          |b, n| {
                              if b.len() == 0 {
                                  format!("{}", n)
                              } else {
                                  format!("{}, {}", b, n)
                              }
                          });
                pandoc.args(&[&format!("--number-offset={}", nums)])
            }

            To(ref f)                => pandoc.args(&["-t", &f.render()]),
            DataDir(dir)             => pandoc.args(&[&format!("--data-dir={}", dir.display())]),
            Strict                   => pandoc.args(&["--strict"]),
            ParseRaw                 => pandoc.args(&["--parse-raw"]),
            Smart                    => pandoc.args(&["--smart"]),
            OldDashes                => pandoc.args(&["--old-dashes"]),
            BaseHeaderLevel(n)       => pandoc.args(&[&format!("--base-header-level={}", n)]),
            IndentedCodeClasses(s)   => pandoc.args(&[&format!("--indented-code-classes={}", s)]),
            Filter(program)          => pandoc.args(&[&format!("--filter={}", program.display())]),
            Normalize                => pandoc.args(&["--normalize"]),
            PreserveTabs             => pandoc.args(&["--preserve-tabs"]),
            TabStop(n)               => pandoc.args(&[&format!("--tab-stop={}", n)]),
            TrackChanges(ref v)      => pandoc.args(&[&format!("--track-changes={}", v)]),
            ExtractMedia(p)          => pandoc.args(&[&format!("--extract-media={}", p.display())]),
            Standalone               => pandoc.args(&["--standalone"]),
            Template(p)              => pandoc.args(&[&format!("--template={}", p.display())]),
            Meta(k, Some(v))         => pandoc.args(&["-M", &format!("{}:{}", k, v)]),
            Meta(k, None)            => pandoc.args(&["-M", k]),
            Var(k, Some(v))          => pandoc.args(&["-V", &format!("{}:{}", k, v)]),
            Var(k, None)             => pandoc.args(&["-V", k]),
            PrintDefaultTemplate(f)  => pandoc.args(&[&format!("--print-default-template={}", f)]),
            PrintDefaultDataFile(f)  => pandoc.args(&[&format!("--print-default-data-file={}", f.display())]),
            NoWrap                   => pandoc.args(&["--no-wrap"]),
            Columns(n)               => pandoc.args(&[&format!("--columns={}", n)]),
            TableOfContents          => pandoc.args(&["--table-of-contents"]),
            TableOfContentsDepth(d)  => pandoc.args(&[&format!("--toc-depth={}", d)]),
            NoHighlight              => pandoc.args(&["--no-highlight"]),
            HighlightStyle(s)        => pandoc.args(&[&format!("--highlight-style={}", s)]),
            IncludeInHeader(p)       => pandoc.args(&[&format!("--include-in-header={}", p.display())]),
            IncludeBeforeBody(p)     => pandoc.args(&[&format!("--include-before-body={}", p.display())]),
            IncludeAfterBody(p)      => pandoc.args(&[&format!("--include-after-body={}", p.display())]),
            SelfContained            => pandoc.args(&["--self-contained"]),
            Offline                  => pandoc.args(&["--offline"]),
            Html5                    => pandoc.args(&["--html5"]),
            HtmlQTags                => pandoc.args(&["--html-q-tags"]),
            Ascii                    => pandoc.args(&["--ascii"]),
            ReferenceLinks           => pandoc.args(&["--reference-links"]),
            AtxHeaders               => pandoc.args(&["--atx-headers"]),
            Chapters                 => pandoc.args(&["--chapters"]),
            NumberSections           => pandoc.args(&["--number-sections"]),
            NoTexLigatures           => pandoc.args(&["--no-tex-ligatures"]),
            Listings                 => pandoc.args(&["--listings"]),
            Incremental              => pandoc.args(&["--incremental"]),
            SlideLevel(n)            => pandoc.args(&[format!("--slide-level={}", n)]),
            SectionDivs              => pandoc.args(&["--section-divs"]),
            DefaultImageExtension(s) => pandoc.args(&[format!("--default-image-extension={}", s)]),
            EmailObfuscation(o)      => pandoc.args(&[format!("--email-obfuscation={}", o)]),
            IdPrefix(s)              => pandoc.args(&[format!("--id-prefix={}", s)]),
            TitlePrefix(s)           => pandoc.args(&[format!("--title-prefix={}", s)]),
            Css(url)                 => pandoc.args(&[format!("--css={}", url)]),
            ReferenceOdt(file)       => pandoc.args(&[format!("--reference-odt={}", file.display())]),
            ReferenceDocx(file)      => pandoc.args(&[&format!("--reference-docx={}", file.display())]),
            EpubStylesheet(file)     => pandoc.args(&[&format!("--epub-stylesheet={}", file.display())]),
            EpubCoverImage(file)     => pandoc.args(&[&format!("--epub-cover-image={}", file.display())]),
            EpubMetadata(file)       => pandoc.args(&[&format!("--epub-metadata={}", file.display())]),
            EpubEmbedFont(file)      => pandoc.args(&[&format!("--epub-embed-font={}", file.display())]),
            EpubChapterLevel(num)    => pandoc.args(&[&format!("--epub-chapter-level={}", num)]),
            LatexEngine(program)     => pandoc.args(&[&format!("--latex-engine={}", program.display())]),
            LatexEngineOpt(s)        => pandoc.args(&[&format!("--latex-engine-opt={}", s)]),
            Bibliography(file)       => pandoc.args(&[&format!("--bibliography={}", file.display())]),
            Csl(file)                => pandoc.args(&[&format!("--csl={}", file.display())]),
            CitationAbbreviations(f) => pandoc.args(&[&format!("--citation-abbreviations={}", f.display())]),
            Natbib                   => pandoc.args(&["--natbib"]),
            Biblatex                 => pandoc.args(&["--biblatex"]),
            LatexMathML(Some(url))   => pandoc.args(&[&format!("--latexmathml={}", url)]),
            AsciiMathML(Some(url))   => pandoc.args(&[&format!("--asciimathml={}", url)]),
            MathML(Some(url))        => pandoc.args(&[&format!("--mathml={}", url)]),
            MimeTex(Some(url))       => pandoc.args(&[&format!("--mimetex={}", url)]),
            WebTex(Some(url))        => pandoc.args(&[&format!("--webtex={}", url)]),
            JsMath(Some(url))        => pandoc.args(&[&format!("--jsmath={}", url)]),
            MathJax(Some(url))       => pandoc.args(&[&format!("--mathjax={}", url)]),
            Katex(Some(url))         => pandoc.args(&[&format!("--katex={}", url)]),
            LatexMathML(None)        => pandoc.args(&["--latexmathml"]),
            AsciiMathML(None)        => pandoc.args(&["--asciimathml"]),
            MathML(None)             => pandoc.args(&["--mathml"]),
            MimeTex(None)            => pandoc.args(&["--mimetex["]),
            WebTex(None)             => pandoc.args(&["--webtex["]),
            JsMath(None)             => pandoc.args(&["--jsmath["]),
            MathJax(None)            => pandoc.args(&["--mathjax["]),
            Katex(None)              => pandoc.args(&["--katex["]),
            KatexStylesheet(url)     => pandoc.args(&[&format!("--katex-stylesheet={}", url)]),
            GladTex                  => pandoc.args(&["--gladtex"]),
            Trace                    => pandoc.args(&["--trace"]),
            DumpArgs                 => pandoc.args(&["--dump-args"]),
            IgnoreArgs               => pandoc.args(&["--ignore-args"]),
            Verbose                  => pandoc.args(&["--verbose"]),
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

pub use DocumentClass::*;

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

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use OutputFormat::*;
        match *self {
            Native => write!(fmt, "native"),
            Json => write!(fmt, "json"),
            Plain => write!(fmt, "plain"),
            Markdown => write!(fmt, "markdown"),
            MarkdownStrict => write!(fmt, "markdown_strict"),
            MarkdownPhpextra => write!(fmt, "markdown_phpextra"),
            MarkdownGithub => write!(fmt, "markdown_github"),
            Commonmark => write!(fmt, "commonmark"),
            Rst => write!(fmt, "rst"),
            Html => write!(fmt, "html"),
            Html5 => write!(fmt, "html5"),
            Latex => write!(fmt, "latex"),
            Beamer => write!(fmt, "beamer"),
            Context => write!(fmt, "context"),
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

impl std::fmt::Display for InputFormat {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use InputFormat::*;
        match *self {
            Native => write!(fmt, "native"),
            Json => write!(fmt, "json"),
            Markdown => write!(fmt, "markdown"),
            MarkdownStrict => write!(fmt, "markdown_strict"),
            MarkdownPhpextra => write!(fmt, "markdown_phpextra"),
            MarkdownGithub => write!(fmt, "markdown_github"),
            Commonmark => write!(fmt, "commonmark"),
            Rst => write!(fmt, "rst"),
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
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum MarkdownExtension {
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

impl std::fmt::Display for MarkdownExtension {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use MarkdownExtension::*;
        match *self {
            EscapedLineBreaks => write!(fmt, "escaped_line_breaks"),
            BlankBeforeHeader => write!(fmt, "blank_before_header"),
            HeaderAttributes => write!(fmt, "header_attributes"),
            AutoIdentifiers => write!(fmt, "auto_identifiers"),
            ImplicitHeaderReferences => write!(fmt, "implicit_header_references"),
            BlankBeforeBlockQuote => write!(fmt, "blank_before_block_quote"),
            FencedCodeBlocks => write!(fmt, "fenced_code_blocks"),
            BacktickCodeBlocks => write!(fmt, "backtick_code_blocks"),
            FencedCodeAttributes => write!(fmt, "fenced_code_attributes"),
            LineBlocks => write!(fmt, "line_blocks"),
            FancyLists => write!(fmt, "fancy_lists"),
            Startnum => write!(fmt, "startnum"),
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
            RawHtml => write!(fmt, "raw_html"),
            MarkdownInHtmlBlocks => write!(fmt, "markdown_in_html_blocks"),
            NativeDivs => write!(fmt, "native_divs"),
            NativeSpans => write!(fmt, "native_spans"),
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
        }
    }
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
pub struct Pandoc<'a> {
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
    options: Vec<PandocOption<'a>>,
}

use std::convert::Into;
use std::borrow::Cow;

/// does nothing useful, simply gives you a builder object
/// convenience function so you can call pandoc::new()
pub fn new<'a>() -> Pandoc<'a> { Default::default() }

impl<'b> Pandoc<'b> {
    /// does nothing useful, simply gives you a builder object
    pub fn new() -> Pandoc<'b> { Default::default() }
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

    pub fn add_option(&mut self, option: PandocOption<'b>) {
        self.options.push(option);
    }

    fn run(self) -> Result<Vec<u8>, PandocError> {
        let mut cmd = Command::new("pandoc");
        if let Some(format) = self.output_format {
            cmd.arg("-t").arg(format.to_string());
        }
        if let Some(format) = self.input_format {
            cmd.arg("-f").arg(format.to_string());
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
            cmd.arg("--variable").arg(format!("documentclass={}", doc_class.to_string()));
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
        for opt in self.options {
            opt.apply(&mut cmd);
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
        let format = self.output_format.as_ref().map(ToString::to_string).unwrap();
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
            PandocError::PandocNotFound => write!(fmt, "Pandoc not found, did you forget to install pandoc?"),
        }
    }
}

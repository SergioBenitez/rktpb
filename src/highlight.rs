use std::io;

use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Theme};

pub const HIGHLIGHT_EXTS: &[&str] = &[
    "Appfile", "Appraisals", "Berksfile", "Brewfile", "C", "Cheffile", "DOT", "Deliverfile",
    "Emakefile", "Fastfile", "GNUmakefile", "Gemfile", "Guardfile", "M", "Makefile",
    "OCamlMakefile", "PL", "R", "Rakefile", "Rantfile", "Rprofile", "S", "SConscript",
    "SConstruct", "Scanfile", "Sconstruct", "Snakefile", "Snapfile", "Thorfile", "Vagrantfile",
    "adp", "applescript", "as", "asa", "asp", "bash", "bat", "bib", "bsh", "build", "builder", "c",
    "c++", "capfile", "cc", "cgi", "cl", "clisp", "clj", "cls", "cmd", "config.ru", "cp", "cpp",
    "cpy", "cs", "css", "css.erb", "css.liquid", "csx", "cxx", "d", "ddl", "di", "diff", "dml",
    "dot", "dpr", "dtml", "el", "emakefile", "erb", "erbsql", "erl", "fasl", "fcgi", "fish",
    "gemspec", "go", "gradle", "groovy", "gv", "gvy", "gyp", "gypi", "h", "h++", "haml", "hh",
    "hpp", "hrl", "hs", "htc", "htm", "html", "html.erb", "hxx", "inc", "inl", "ipp", "irbrc",
    "java", "jbuilder", "js", "js.erb", "json", "jsp", "l", "lhs", "lisp", "lsp", "ltx", "lua",
    "m", "mak", "make", "makefile", "markdn", "markdown", "matlab", "md", "mdown", "mk", "ml",
    "mli", "mll", "mly", "mm", "mud", "opml", "p", "pas", "patch", "php", "php3", "php4", "php5",
    "php7", "phps", "phpt", "phtml", "pl", "pm", "pod", "podspec", "prawn", "properties", "pxd",
    "pxd.in", "pxi", "pxi.in", "py", "py3", "pyi", "pyw", "pyx", "pyx.in", "r", "rabl", "rails",
    "rake", "rb", "rbx", "rd", "re", "rest", "rhtml", "rjs", "rpy", "rs", "rss", "rst",
    "ruby.rail", "rxml", "s", "sass", "sbt", "scala", "scm", "sconstruct", "script editor", "sh",
    "shtml", "simplecov", "sql", "sql.erb", "ss", "sty", "svg", "t", "tcl", "tex", "textile",
    "thor", "tld", "tmpl", "tpl", "txt", "wscript", "xhtml", "xml", "xsd", "xslt", "yaml", "yaws",
    "yml", "zsh",
];

pub struct Highlighter {
    theme: Theme,
    syntaxes: SyntaxSet,
}

impl Highlighter {
    pub fn default() -> Option<Self> {
        let mut reader = io::Cursor::new(include_str!("../static/GitHub.tmtheme"));
        Some(Highlighter {
            theme: ThemeSet::load_from_reader(&mut reader).ok()?,
            syntaxes: SyntaxSet::load_defaults_newlines(),
        })
    }

    pub fn contains(ext: &str) -> bool {
        HIGHLIGHT_EXTS.binary_search(&ext).is_ok()
    }

    pub fn highlight(&self, code: &str, lang: &str) -> io::Result<String> {
        let syntax = self.syntaxes.find_syntax_by_token(lang)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing syntax"))?;

        syntect::html::highlighted_html_for_string(code, &self.syntaxes, syntax, &self.theme)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn render_markdown(&self, markdown: &str) -> io::Result<String> {
        let options = comrak::ComrakOptions {
            extension: {
                let mut opts = comrak::ComrakExtensionOptions::default();
                opts.strikethrough = true;
                opts.tagfilter = true;
                opts.table = true;
                opts.autolink = true;
                opts.tasklist = true;
                opts.superscript = true;
                opts.footnotes = true;
                opts.front_matter_delimiter = Some("---".into());
                opts.header_ids = Some(String::new());
                opts
            },
            parse: Default::default(),
            render: {
                let mut opts = comrak::ComrakRenderOptions::default();
                opts.github_pre_lang = true;
                // NB: we use CSP to ensure no JS leaks.
                opts.unsafe_ = true;
                opts
            },
        };

        let arena = comrak::Arena::new();
        let ast = comrak::parse_document(&arena, markdown, &options);
        self.highlight_ast(ast);

        let mut html = Vec::new();
        comrak::format_html(ast, &options, &mut html)?;
        String::from_utf8(html).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn highlight_ast<'a>(&self, ast: &'a comrak::nodes::AstNode<'a>) {
        use comrak::arena_tree::NodeEdge;
        use comrak::nodes::{NodeValue, NodeHtmlBlock};

        for node in ast.traverse() {
            if let NodeEdge::Start(node) = node {
                let mut data = node.data.borrow_mut();
                if let NodeValue::CodeBlock(ref mut block) = data.value {
                    if let Ok(highlighted) = self.highlight(&block.literal, &block.info) {
                        data.value = NodeValue::HtmlBlock(NodeHtmlBlock {
                            literal: highlighted,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
}

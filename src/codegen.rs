use crate::ast::Node;

pub fn generate(nodes: &[Node]) -> String {
    let mut out = String::new();

    for node in nodes {
        out.push_str(&gen_node(node));
    }

    out
}

fn gen_node(node: &Node) -> String {
    match node {
        Node::Text(t) => escape_html(t),

        Node::Paragraph(children) => {
            format!("<p>{}</p>", generate(children))
        }

        Node::Heading { level, content } => {
            let tag = format!("h{}", level);
            format!("<{tag}>{}</{tag}>", generate(content))
        }

        Node::Strong(children) => {
            format!("<strong>{}</strong>", generate(children))
        }

        Node::Emphasis(children) => {
            format!("<em>{}</em>", generate(children))
        }

        Node::Strikethrough(children) => {
            format!("<del>{}</del>", generate(children))
        }

        Node::InlineCode(code) => {
            format!("<code>{}</code>", escape_html(code))
        }

        Node::CodeBlock { language, content } => {
            let rendered = highlight_code(language.as_deref(), content);

            match language.as_deref() {
                Some(lang) => format!(
                    "<pre><code class=\"language-{}\">{}</code></pre>",
                    lang,
                    rendered
                ),
                None => format!(
                    "<pre><code>{}</code></pre>",
                    rendered
                ),
            }
        }

        Node::BlockQuote(children) => {
            format!("<blockquote>{}</blockquote>", generate(children))
        }

        Node::UnorderedList(items) => {
            let mut out = String::from("<ul>");

            for item in items {
                out.push_str("<li>");
                out.push_str(&generate(item));
                out.push_str("</li>");
            }

            out.push_str("</ul>");
            out
        }

        Node::OrderedList(items) => {
            let mut out = String::from("<ol>");

            for item in items {
                out.push_str("<li>");
                out.push_str(&generate(item));
                out.push_str("</li>");
            }

            out.push_str("</ol>");
            out
        }

        Node::Link { text, url, .. } => {
            format!("<a href=\"{}\">{}</a>", escape_attr(url), generate(text))
        }

        Node::Image { alt, url, .. } => {
            format!(
                "<img src=\"{}\" alt=\"{}\" />",
                escape_attr(url),
                escape_attr(alt)
            )
        }

        Node::SoftBreak => "<br>".to_string(),
        Node::LineBreak => "<br>".to_string(),

        Node::Html(raw) => raw.clone(),
        Node::EscapedChar(c) => c.to_string(),

        Node::HorizontalRule => "<hr>".to_string(),

        Node::Table { .. } => "<table>TODO</table>".to_string(),
    }
}

fn escape_html(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

fn escape_attr(text: &str) -> String {
    text.replace("\"", "&quot;")
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

fn highlight_code(language: Option<&str>, content: &str) -> String {
    let language = language.unwrap_or("").to_ascii_lowercase();

    match language.as_str() {
        "rust" => highlight_language(content, &RUST_KEYWORDS, Some('/'), true, false),
        "js" | "javascript" => highlight_language(content, &JS_KEYWORDS, Some('/'), true, true),
        "py" | "python" => highlight_language(content, &PYTHON_KEYWORDS, Some('#'), false, false),
        _ => escape_html(content),
    }
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "mut", "pub",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "async", "await", "dyn", "move", "ref", "mut",
];

const JS_KEYWORDS: &[&str] = &[
    "await", "break", "case", "catch", "class", "const", "continue", "debugger", "default",
    "delete", "do", "else", "export", "extends", "finally", "for", "function", "if",
    "import", "in", "instanceof", "let", "new", "return", "super", "switch", "this",
    "throw", "try", "typeof", "var", "void", "while", "with", "yield",
];

const PYTHON_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
    "elif", "else", "except", "False", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise", "return",
    "True", "try", "while", "with", "yield",
];

fn highlight_language(
    code: &str,
    keywords: &[&str],
    line_comment_start: Option<char>,
    allow_block_comments: bool,
    allow_backticks: bool,
) -> String {
    let mut out = String::new();
    let mut chars = code.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '/' if line_comment_start == Some('/') && chars.peek() == Some(&'/') => {
                out.push_str("<span class=\"token comment\">//");
                chars.next();
                while let Some(&next) = chars.peek() {
                    let next = next;
                    chars.next();
                    out.push_str(&escape_html(&next.to_string()));
                    if next == '\n' {
                        break;
                    }
                }
                out.push_str("</span>");
            }
            '#' if line_comment_start == Some('#') => {
                out.push_str("<span class=\"token comment\">#");
                while let Some(&next) = chars.peek() {
                    let next = next;
                    chars.next();
                    out.push_str(&escape_html(&next.to_string()));
                    if next == '\n' {
                        break;
                    }
                }
                out.push_str("</span>");
            }
            '/' if allow_block_comments && chars.peek() == Some(&'*') => {
                out.push_str("<span class=\"token comment\">/*");
                chars.next();
                while let Some(next) = chars.next() {
                    out.push_str(&escape_html(&next.to_string()));
                    if next == '*' && chars.peek() == Some(&'/') {
                        out.push_str("/");
                        chars.next();
                        break;
                    }
                }
                out.push_str("</span>");
            }
            '\"' | '\'' | '`' if ch == '\"' || ch == '\'' || (allow_backticks && ch == '`') => {
                out.push_str("<span class=\"token string\">");
                out.push_str(&escape_html(&ch.to_string()));
                let terminator = ch;
                while let Some(next) = chars.next() {
                    out.push_str(&escape_html(&next.to_string()));
                    if next == '\\' {
                        if let Some(escaped) = chars.next() {
                            out.push_str(&escape_html(&escaped.to_string()));
                        }
                    } else if next == terminator {
                        break;
                    }
                }
                out.push_str("</span>");
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut ident = c.to_string();
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        ident.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if keywords.contains(&ident.as_str()) {
                    out.push_str("<span class=\"token keyword\">");
                    out.push_str(&escape_html(&ident));
                    out.push_str("</span>");
                } else {
                    out.push_str(&escape_html(&ident));
                }
            }
            c if c.is_ascii_digit() => {
                let mut number = c.to_string();
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == '_' || next == '.' || next == 'x' || next == 'b' || next == 'o' || next.is_ascii_hexdigit() {
                        number.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                out.push_str("<span class=\"token number\">");
                out.push_str(&escape_html(&number));
                out.push_str("</span>");
            }
            other => {
                out.push_str(&escape_html(&other.to_string()));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Node;

    #[test]
    fn rust_code_block_generates_highlight_spans() {
        let nodes = vec![Node::CodeBlock {
            language: Some("rust".into()),
            content: "let x = 42; // answer".into(),
        }];
        let html = generate(nodes.as_slice());

        assert!(html.contains("class=\"language-rust\""));
        assert!(html.contains("<span class=\"token keyword\">let</span>"));
        assert!(html.contains("<span class=\"token number\">42</span>"));
        assert!(html.contains("<span class=\"token comment\">// answer</span>"));
    }
}

pub fn generate_html(ast: Vec<Node>) -> String {
    generate(ast.as_slice())
}

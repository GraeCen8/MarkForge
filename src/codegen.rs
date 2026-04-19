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

        Node::CodeBlock {
            language: _,
            content,
        } => {
            format!("<pre><code>{}</code></pre>", escape_html(content))
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

pub fn generate_html(ast: Vec<Node>) -> String {
    generate(ast.as_slice())
}

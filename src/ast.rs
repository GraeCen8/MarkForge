use crate::token::LexTok;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // ===== Block-level =====
    Heading {
        level: u8,
        content: Vec<Node>,
    },
    Paragraph(Vec<Node>),
    BlockQuote(Vec<Node>),
    UnorderedList(Vec<Vec<Node>>), // list of items
    OrderedList(Vec<Vec<Node>>),
    CodeBlock {
        language: Option<String>,
        content: String,
    },
    HorizontalRule,
    Table {
        headers: Vec<Vec<Node>>,
        rows: Vec<Vec<Vec<Node>>>,
    },
    // ===== Inline-level =====
    Text(String),
    Emphasis(Vec<Node>),      // *italic*
    Strong(Vec<Node>),        // **bold**
    Strikethrough(Vec<Node>), // ~~text~~
    InlineCode(String),
    Link {
        text: Vec<Node>,
        url: String,
        title: Option<String>,
    },
    Image {
        alt: String,
        url: String,
        title: Option<String>,
    },
    // ===== Special / misc =====
    LineBreak,
    SoftBreak,
    Html(String),
    EscapedChar(char),
}

pub struct Parser {
    tokens: Vec<LexTok>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<LexTok>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();

        while !self.is_eof() {
            if matches!(self.peek(), LexTok::Newline) {
                self.advance();
                continue;
            }
            nodes.push(self.parse_block());
        }

        nodes
    }
}

//these are the main parsing funcs that parses a given thing
impl Parser {
    fn parse_block(&mut self) -> Node {
        match self.peek() {
            LexTok::Hash => self.parse_heading(),
            LexTok::Dash | LexTok::Number(_) => self.parse_list(),
            LexTok::GreaterThan => self.parse_blockquote(),
            LexTok::Backtick => {
                if self.is_code_block_start() {
                    self.parse_code_block()
                } else {
                    self.parse_code_inline()
                }
            }
            _ => self.parse_paragraph(),
        }
    }

    fn parse_inline(&mut self) -> Node {
        match self.peek() {
            LexTok::Text(t) => {
                let t = t.clone();
                self.advance();
                Node::Text(t)
            }

            LexTok::Star => {
                if *self.peek_next() == LexTok::Star {
                    self.parse_strong()
                } else {
                    self.parse_emphasis()
                }
            }

            LexTok::Backtick => self.parse_code_inline(),
            LexTok::LBracket => self.parse_link(),

            LexTok::Tilde => {
                if *self.peek_next() == LexTok::Tilde {
                    self.parse_strikethrough()
                } else {
                    self.advance();
                    Node::Text("~".into())
                }
            }

            LexTok::Newline => {
                self.advance();
                Node::SoftBreak
            }

            _ => self.parse_text_token(),
        }
    }
}

//here are the helper functions
impl Parser {
    fn peek(&self) -> &LexTok {
        self.tokens.get(self.pos).unwrap()
    }

    fn peek_next(&self) -> &LexTok {
        self.tokens.get(self.pos + 1).unwrap()
    }

    fn advance(&mut self) -> Option<LexTok> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    fn is_code_block_start(&self) -> bool {
        matches!(
            (
                self.peek(),
                self.tokens.get(self.pos + 1),
                self.tokens.get(self.pos + 2)
            ),
            (LexTok::Backtick, Some(LexTok::Backtick), Some(LexTok::Backtick))
        )
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(), LexTok::EOF)
    }

    fn consume_newlines(&mut self) {
        if matches!(self.peek(), LexTok::Newline) {
            self.advance();
        }
    }

    fn parse_text_token(&mut self) -> Node {
        if let Some(tok) = self.advance() {
            Node::Text(self.token_to_text(&tok))
        } else {
            Node::Text(String::new())
        }
    }

    fn token_to_text(&self, tok: &LexTok) -> String {
        match tok {
            LexTok::Text(t) => t.clone(),
            LexTok::Hash => "#".into(),
            LexTok::Star => "*".into(),
            LexTok::Underscore => "_".into(),
            LexTok::Tilde => "~".into(),
            LexTok::Backtick => "`".into(),
            LexTok::Dash => "-".into(),
            LexTok::Plus => "+".into(),
            LexTok::Number(n) => n.clone(),
            LexTok::LBracket => "[".into(),
            LexTok::RBracket => "]".into(),
            LexTok::LParen => "(".into(),
            LexTok::RParen => ")".into(),
            LexTok::GreaterThan => ">".into(),
            LexTok::Pipe => "|".into(),
            LexTok::Newline => "\n".into(),
            LexTok::EOF => String::new(),
        }
    }

    fn tokens_to_text(&self, start: usize, end: usize) -> String {
        self.tokens[start..end]
            .iter()
            .map(|tok| self.token_to_text(tok))
            .collect()
    }
}

//now we need to build the parsing functions for the specific things like a paragraph or header
impl Parser {
    fn parse_paragraph(&mut self) -> Node {
        let mut content = Vec::new();

        while !self.is_eof() {
            match self.peek() {
                LexTok::Newline => {
                    self.advance();
                    break;
                }
                _ => content.push(self.parse_inline()),
            }
        }

        Node::Paragraph(content)
    }

    fn parse_heading(&mut self) -> Node {
        let mut level = 0;

        while matches!(self.peek(), LexTok::Hash) {
            self.advance();
            level += 1;
        }

        let mut content = Vec::new();

        while !matches!(self.peek(), LexTok::Newline | LexTok::EOF) {
            content.push(self.parse_inline());
        }

        self.consume_newlines();

        Node::Heading { level, content }
    }

    fn parse_list(&mut self) -> Node {
        let mut items = Vec::new();

        while matches!(self.peek(), LexTok::Dash | LexTok::Number(_)) {
            items.push(self.parse_list_item());
        }

        Node::UnorderedList(items)
    }

    fn parse_blockquote(&mut self) -> Node {
        self.advance();

        let mut content = Vec::new();

        while !matches!(self.peek(), LexTok::Newline | LexTok::EOF) {
            content.push(self.parse_inline());
        }

        self.consume_newlines();

        Node::BlockQuote(content)
    }

    fn parse_link(&mut self) -> Node {
        // example link [link text](http://www.example.com)
        let start_pos = self.pos;
        self.advance();

        let mut text = Vec::new();

        while !matches!(self.peek(), LexTok::RBracket | LexTok::EOF) {
            text.push(self.parse_inline());
        }

        if matches!(self.peek(), LexTok::RBracket) {
            self.advance();
            if matches!(self.peek(), LexTok::LParen) {
                self.advance();
                let mut url = String::new();
                while !matches!(self.peek(), LexTok::RParen | LexTok::EOF) {
                    if let Some(LexTok::Text(t)) = self.advance() {
                        url.push_str(&t);
                    } else if let Some(tok) = self.advance() {
                        url.push_str(&self.token_to_text(&tok));
                    }
                }
                self.advance();
                return Node::Link {
                    text,
                    url,
                    title: None,
                };
            }
        }

        // Not an actual link, treat as literal text
        let literal = self.tokens_to_text(start_pos, self.pos);
        Node::Text(literal)
    }
}

// this is where all the inline impls go
impl Parser {
    fn parse_list_item(&mut self) -> Vec<Node> {
        self.advance();

        let mut content = Vec::new();

        while !matches!(self.peek(), LexTok::Newline | LexTok::EOF) {
            content.push(self.parse_inline());
        }

        self.consume_newlines();

        content
    }

    fn parse_strong(&mut self) -> Node {
        self.advance(); // *
        self.advance(); // * skip through both we know they are there 

        let mut content = Vec::new();

        while !matches!(self.peek(), LexTok::Star | LexTok::EOF) {
            content.push(self.parse_inline())
        }

        self.advance();
        self.advance();

        Node::Strong(content)
    }

    fn parse_emphasis(&mut self) -> Node {
        self.advance(); // *

        let mut content = Vec::new();

        while !matches!(self.peek(), LexTok::Star | LexTok::EOF) {
            content.push(self.parse_inline())
        }

        self.advance();

        Node::Emphasis(content)
    }

    fn parse_strikethrough(&mut self) -> Node {
        self.advance();
        self.advance();

        let mut content = Vec::new();

        while !matches!(self.peek(), LexTok::Tilde | LexTok::EOF) {
            content.push(self.parse_inline())
        }

        self.advance();
        self.advance();

        Node::Strikethrough(content)
    }

    fn parse_code_inline(&mut self) -> Node {
        self.advance();

        let mut content = String::new();

        while !matches!(self.peek(), LexTok::Backtick | LexTok::EOF) {
            if let Some(tok) = self.advance() {
                content.push_str(&self.token_to_text(&tok));
            }
        }

        self.advance();

        Node::InlineCode(content)
    }

    fn parse_code_block(&mut self) -> Node {
        self.advance();
        self.advance();
        self.advance();

        //an optional language
        let language = match self.peek() {
            LexTok::Text(t) => {
                let lang = t.clone();
                self.advance();
                Some(lang)
            }
            _ => None,
        };

        let mut content = String::new();

        while !self.is_code_block_end() && !self.is_eof() {
            if let Some(tok) = self.advance() {
                content.push_str(&self.token_to_text(&tok));
            }
        }

        self.advance();
        self.advance();
        self.advance();

        Node::CodeBlock { language, content }
    }

    fn is_code_block_end(&self) -> bool {
        matches!(
            (
                self.peek(),
                self.tokens.get(self.pos + 1),
                self.tokens.get(self.pos + 2)
            ),
            (LexTok::Backtick, Some(LexTok::Backtick), Some(LexTok::Backtick))
        )
    }
}

pub fn parse_tokens(tokens: Vec<LexTok>) -> Vec<Node> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::lex_text;

    #[test]
    fn code_block_preserves_special_characters() {
        let markdown = "```rust\nlet x = [1, 2];\nprintln!(\"{}\\n\", x[0]);\n```\n";
        let ast = parse_tokens(lex_text(&markdown.to_string()));

        assert_eq!(ast.len(), 1);
        if let Node::CodeBlock { language, content } = &ast[0] {
            assert_eq!(language.as_deref(), Some("rust"));
            assert!(content.contains("[1, 2]"));
            assert!(content.contains("println!(\"{}\\n\", x[0]);"));
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn inline_code_preserves_brackets_and_parens() {
        let markdown = "Use `foo(bar)` and `[baz]` in code.";
        let ast = parse_tokens(lex_text(&markdown.to_string()));

        assert_eq!(ast.len(), 1);
        if let Node::Paragraph(children) = &ast[0] {
            let code_node = children.iter().find_map(|node| {
                if let Node::InlineCode(code) = node {
                    Some(code)
                } else {
                    None
                }
            });
            assert_eq!(code_node.map(|c| c.as_str()), Some("foo(bar)"));
        } else {
            panic!("Expected Paragraph");
        }
    }

    #[test]
    fn raw_text_preserves_unhandled_punctuation() {
        let markdown = "This is > a line with [brackets] and (parentheses).";
        let ast = parse_tokens(lex_text(&markdown.to_string()));

        assert_eq!(ast.len(), 1);
        if let Node::Paragraph(children) = &ast[0] {
            let rendered: String = children
                .iter()
                .map(|node| match node {
                    Node::Text(t) => t.clone(),
                    _ => String::new(),
                })
                .collect();
            assert_eq!(rendered, "This is > a line with [brackets] and (parentheses).",);
        } else {
            panic!("Expected Paragraph");
        }
    }
}

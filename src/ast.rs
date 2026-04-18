use crate::token::{LexTok, Tokenizer};

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
            nodes.push(self.parse_block());
        }

        nodes
    }
}

//these are the main parsing funcs that parses a given thing
impl Parser {
    fn parse_block(&mut self) -> Node {
        let next_tok = self.peek();
        match next_tok {
            LexTok::Hash => self.parse_heading(),
            LexTok::Dash | LexTok::Number(_) => self.parse_list(),
            LexTok::GreaterThan => self.parse_blockquote(),
            LexTok::Backtick => {
                if self.peek() == LexTok::Backtick {
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
                if self.peek_next() == LexTok::Star {
                    self.parse_strong()
                } else {
                    self.parse_emphasis()
                }
            }

            LexTok::Backtick => self.parse_code_inline(),
            LexTok::LBracket => self.parse_link(),

            LexTok::Tilde => {
                if self.peek_next() == LexTok::Tilde {
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

            _ => {
                self.advance();
                Node::Text("".into())
            }
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

    fn is_eof(&self) -> bool {
        matches!(self.peek(), Some(LexTok::EOF))
    }

    fn consume_newlines(&mut self) {
        if matches!(self.peek(), LexTok::Newline) {
            self.advance();
        }
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

        while !matches!(self.peek(), Some(LexTok::Newline) | Some(LexTok::EOF)) {
            content.push(self.parse_inline());
        }

        self.consume_newlines();

        Node::BlockQuote(content)
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
            match self.advance() {
                Some(LexTok::Text(t)) => content.push_str(&t),
                Some(LexTok::Newline) => content.push('\n'),
                _ => {}
            }
        }

        self.advance();
        self.advance();
        self.advance();

        Node::CodeBlock { language, content }
    }

    fn parse_link(&mut self) -> Node {
        // example link [link text](http://www.example.com)
        self.advance();

        let mut text = Vec::new();

        while !matches!(self.peek(), LexTok::RBracket | LexTok::EOF) {
            text.push(self.parse_inline());
        }

        let mut url = String::new();

        while !matches!(self.peek(), LexTok::RParen | LexTok::EOF) {
            if let Some(LexTok::Text(t)) = self.advance() {
                url.push_str(&t);
            }
        }

        self.advance();
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
            if let Some(LexTok::Text(t)) = self.advance() {
                content.push_str(&t);
            }
        }

        self.advance();

        Node::InlineCode(content)
    }

    fn is_code_block_end(&mut self) -> bool {
        matches!(
            (self.peek(), self.peek_next()),
            (LexTok::Backtick, LexTok::Backtick)
        )
    }
}

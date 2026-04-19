// this is the tokenizer that takes the markdown and turns it into a list of tokens

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum LexTok {
    // structural markers
    Hash,           // #
    Star,           // *
    Underscore,     // _
    Tilde,          // ~
    Backtick,       // `
    Dash,           // -
    Plus,           // +
    Number(String), // 1. 2. etc

    LBracket, // [
    RBracket, // ]
    LParen,   // (
    RParen,   // )

    GreaterThan, // >
    Pipe,        // |

    Newline,
    Text(String),

    EOF,
}

pub struct Tokenizer {
    input: Vec<char>,
    pos: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<LexTok> {
        let mut tokens: Vec<LexTok> = Vec::new();

        while let Some(c) = self.peek() {
            tokens.push(self.match_tok(c))
        }

        tokens.push(LexTok::EOF);
        tokens
    }

    fn match_tok(&mut self, c: char) -> LexTok {
        match c {
            '#' => {
                self.advance();
                LexTok::Hash
            }
            '*' => {
                self.advance();
                LexTok::Star
            }
            '_' => {
                self.advance();
                LexTok::Underscore
            }
            '~' => {
                self.advance();
                LexTok::Tilde
            }
            '`' => {
                self.advance();
                LexTok::Backtick
            }
            '-' => {
                self.advance();
                LexTok::Dash
            }
            '+' => {
                self.advance();
                LexTok::Plus
            }
            '[' => {
                self.advance();
                LexTok::LBracket
            }
            ']' => {
                self.advance();
                LexTok::RBracket
            }
            '(' => {
                self.advance();
                LexTok::LParen
            }
            ')' => {
                self.advance();
                LexTok::RParen
            }
            '>' => {
                self.advance();
                LexTok::GreaterThan
            }
            '|' => {
                self.advance();
                LexTok::Pipe
            }
            '\n' => {
                self.advance();
                LexTok::Newline
            }

            c if c.is_numeric() => self.read_number(),

            _ => self.read_text(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn read_text(&mut self) -> LexTok {
        let start = self.pos;

        while let Some(c) = self.peek() {
            if "#*_~`_+[]()>\n|".contains(c) {
                break;
            }
            self.advance();
        }

        let text: String = self.input[start..self.pos].iter().collect();
        LexTok::Text(text)
    }

    fn read_number(&mut self) -> LexTok {
        let start = self.pos;

        while let Some(c) = self.peek() {
            if c.is_numeric() {
                self.advance();
            } else {
                break;
            }
        }

        let num: String = self.input[start..self.pos].iter().collect();
        LexTok::Number(num)
    }
}

pub fn lex_text(text: &String) -> Vec<LexTok> {
    let mut tokenizer = Tokenizer::new(text);
    tokenizer.tokenize()
}

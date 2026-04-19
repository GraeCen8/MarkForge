# Building a Programming Language with Rust + Cranelift

## What is Cranelift?

Cranelift is a fast, Rust-native code generator used by Wasmtime and rustc (as a backend). Unlike LLVM, it's:

- **Written in Rust** — no C++ FFI headaches
- **Fast to compile** — great for JIT use cases
- **Simpler API** — less powerful optimizer, but far easier to get started with
- **No external install** — just a cargo dependency

---

## Project Setup

```toml
# Cargo.toml
[dependencies]
cranelift = "0.115"
cranelift-module = "0.115"
cranelift-object = "0.115"   # for AOT: emit .o files
cranelift-native = "0.115"   # detects your CPU's ISA
target-lexicon = "0.12"
```

---

## The Pipeline

```
Source code  →  Lexer  →  Parser  →  AST  →  Codegen  →  Cranelift IR  →  Native binary
```

You write everything up to and including codegen. Cranelift handles the rest.

---

## Phase 1: Lexer

The lexer converts raw text into a flat list of tokens.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    Ident(String),
    // Keywords
    Fn, Let, Return, If, Else,
    // Operators
    Plus, Minus, Star, Slash,
    Equals, DoubleEq, Lt, Gt,
    // Delimiters
    LParen, RParen, LBrace, RBrace, Comma, Semicolon,
    EOF,
}
```

Walk character by character, match patterns, emit tokens. Keywords are just identifiers you check after the fact:

```rust
fn ident_or_keyword(s: String) -> Token {
    match s.as_str() {
        "fn"     => Token::Fn,
        "let"    => Token::Let,
        "return" => Token::Return,
        "if"     => Token::If,
        "else"   => Token::Else,
        _        => Token::Ident(s),
    }
}
```

---

## Phase 2: AST

Define a tree of nodes that represent your language's structure.

```rust
pub enum Expr {
    Number(i64),
    Ident(String),
    BinOp { op: Op, left: Box<Expr>, right: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
    If { cond: Box<Expr>, then: Vec<Stmt>, else_: Option<Vec<Stmt>> },
}

pub enum Stmt {
    Let { name: String, value: Expr },
    Return(Expr),
    Expr(Expr),
}

pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}
```

---

## Phase 3: Parser

A **recursive descent parser** — one function per grammar rule. It consumes tokens and builds the AST.

```rust
// Parses: let x = <expr>;
fn parse_let(&mut self) -> Stmt {
    self.expect(Token::Let);
    let name = self.expect_ident();
    self.expect(Token::Equals);
    let value = self.parse_expr();
    self.expect(Token::Semicolon);
    Stmt::Let { name, value }
}

// Parses binary expressions with precedence climbing
fn parse_expr(&mut self) -> Expr {
    let mut left = self.parse_primary();
    while let Some(op) = self.peek_binop() {
        self.advance();
        let right = self.parse_primary();
        left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
    }
    left
}
```

---

## Phase 4: Cranelift Codegen

This is where Cranelift enters. You build functions using a `FunctionBuilder`, emitting instructions block by block.

### Setup

```rust
use cranelift::prelude::*;
use cranelift_module::{Module, Linkage};
use cranelift_object::{ObjectModule, ObjectBuilder};

// Detect native ISA (your CPU)
let isa = cranelift_native::builder()
    .unwrap()
    .finish(settings::Flags::new(settings::builder()))
    .unwrap();

let obj_builder = ObjectBuilder::new(
    isa,
    "my_module",
    cranelift_module::default_libcall_names(),
).unwrap();

let mut module = ObjectModule::new(obj_builder);
```

### Declaring a Function

```rust
let mut sig = module.make_signature();
sig.params.push(AbiParam::new(types::I64));  // one i64 argument
sig.returns.push(AbiParam::new(types::I64)); // returns i64

let func_id = module
    .declare_function("add_one", Linkage::Export, &sig)
    .unwrap();
```

### Building the Function Body

```rust
let mut ctx = module.make_context();
ctx.func.signature = sig;

let mut func_ctx = FunctionBuilderContext::new();
let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

// Every function needs at least one basic block
let entry_block = builder.create_block();
builder.append_block_params_for_function_params(entry_block);
builder.switch_to_block(entry_block);
builder.seal_block(entry_block);

// Get the function parameter
let arg = builder.block_params(entry_block)[0];

// Emit: arg + 1
let one = builder.ins().iconst(types::I64, 1);
let result = builder.ins().iadd(arg, one);
builder.ins().return_(&[result]);

builder.finalize();
module.define_function(func_id, &mut ctx).unwrap();
```

### Emit the Object File

```rust
let product = module.finish();
let bytes = product.emit().unwrap();
std::fs::write("output.o", bytes).unwrap();
```

Then link with: `cc output.o -o my_program`

---

## Key Cranelift Concepts

### Basic Blocks

All code lives in **basic blocks** — straight-line sequences of instructions with no internal branching. Branches only happen at the end of a block.

```rust
let then_block = builder.create_block();
let else_block = builder.create_block();
let merge_block = builder.create_block();

// Branch at end of current block
builder.ins().brif(cond, then_block, &[], else_block, &[]);
```

### SSA Values

Every instruction returns a `Value`. Values are immutable — you never overwrite them. For mutable variables, use **stack slots**:

```rust
// Mutable variable pattern
let slot = builder.create_sized_stack_slot(StackSlotData::new(
    StackSlotKind::ExplicitSlot, 8, 0  // 8 bytes for i64
));

// Store
builder.ins().stack_store(value, slot, 0);

// Load
let loaded = builder.ins().stack_load(types::I64, slot, 0);
```

# MarkForge

A fast, lightweight Markdown to HTML converter written in Rust. MarkForge parses Markdown files and generates clean, styled HTML output with support for common Markdown features including headings, lists, code blocks, links, images, and more.

## Features

- **Fast parsing**: Efficient lexer and parser implementation
- **Complete Markdown support**: Headings, paragraphs, lists, code blocks, links, images, blockquotes, tables, and inline formatting
- **Styled HTML output**: Includes CSS styling for immediate readability
- **Command-line interface**: Easy-to-use CLI with input/output file options
- **Rust-native**: No external dependencies for core functionality

## Installation

### Prerequisites

- Rust 1.70 or later (edition 2024)

### Building from source

```bash
git clone <repository-url>
cd MarkForge
cargo build --release
```

The binary will be available at `target/release/MarkForge`.

## Usage

### Basic usage

Convert a Markdown file to HTML:

```bash
./MarkForge -i input.md
```

This will output the HTML to stdout.

### Save to file

```bash
./MarkForge -i input.md -o output.html
```

### Command-line options

- `-i, --input <FILE>`: Input Markdown file (required)
- `-o, --output <FILE>`: Output HTML file (optional, defaults to stdout)
- `-h, --help`: Display help information

## Examples

### Input Markdown (`example.md`)

```markdown
# Hello World

This is a **bold** and *italic* text.

## Code Example

```rust
fn main() {
    println!("Hello, MarkForge!");
}
```

- List item 1
- List item 2

[Link to Rust](https://www.rust-lang.org/)

```

### Output HTML

```bash
./MarkForge -i example.md -o example.html
```

The generated HTML includes embedded CSS for immediate styling and readability.

## Project Structure

```
src/
├── main.rs      # CLI entry point and file I/O
├── token.rs     # Lexer implementation
├── ast.rs       # Abstract Syntax Tree definitions
├── codegen.rs   # HTML code generation
└── styler.rs    # CSS styling and HTML wrapping
```

## Architecture

MarkForge follows a traditional compiler pipeline:

1. **Lexer** (`token.rs`): Converts raw Markdown text into tokens
2. **Parser** (`ast.rs`): Builds an Abstract Syntax Tree from tokens
3. **Code Generator** (`codegen.rs`): Generates HTML from the AST
4. **Styler** (`styler.rs`): Wraps HTML with CSS styling

## Supported Markdown Features

### Block Elements

- Headings (H1-H6)
- Paragraphs
- Blockquotes
- Unordered lists
- Ordered lists
- Code blocks (with optional language highlighting)
- Horizontal rules
- Tables

### Inline Elements

- Bold text (`**text**`)
- Italic text (`*text*`)
- Strikethrough (`~~text~~`)
- Inline code (`code`)
- Links (`[text](url)`)
- Images (`![alt](url)`)

## Development

### Running tests

```bash
cargo test
```

### Building in debug mode

```bash
cargo build
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License.


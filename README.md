# Rusql: A Modern MySQL Client in Rust 🦀

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

A lightning-fast, cross-platform MySQL client written in Rust that provides a familiar interface for MySQL users while leveraging Rust's performance and safety features.

![Screenshot of Rusql](screenshot_placeholder.png)

## ✨ Features

- 🚀 **Cross-Platform Support**: Works on Windows, macOS, and Linux
- 🎨 **Syntax Highlighting**: Beautiful, colorized output for better readability
- 📝 **Command History**: Persistent command history with readline support
- 🔒 **Secure**: Safe password handling and connection management
- 💻 **Familiar Interface**: Similar to the official MySQL client
- ⚡ **Performance**: Built with Rust for optimal speed and memory usage

## 🚀 Quick Start

### Installation

```bash
cargo install rusql
```

Or build from source:

```bash
git clone https://github.com/yourusername/rusql.git
cd rusql
cargo build --release
```

### Basic Usage

Connect to a local MySQL server:
```bash
rusql -u root -p
```

Connect to a remote server:
```bash
rusql -h hostname -P 3306 -u username -p -D database
```

## 🔧 Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `-h, --host` | Host to connect to | localhost |
| `-P, --port` | Port number | 3306 |
| `-u, --user` | Username | None |
| `-p, --password` | Password (will prompt if not provided) | None |
| `-D, --database` | Database to use | None |
| `-e, --execute` | Execute command and quit | None |
| `--no-colors` | Disable colors in output | false |

## 🎯 Features in Detail

### Interactive Mode
- Multi-line query support
- Command history (stored in ~/.mysql_history)
- Tab completion (coming soon)
- Syntax highlighting
- Pretty-printed tables

### Query Execution
- Support for all MySQL query types
- Formatted output for SELECT queries
- Visual feedback for affected rows
- Query timing information
- Error reporting with color highlighting

### Special Commands
- `status`: Show server status
- `clear` or `\c`: Clear screen
- `use [database]`: Switch database
- More coming soon!

## 🛠️ Development

### Prerequisites
- Rust (1.70.0 or later)
- MySQL/MariaDB server (for testing)
- OpenSSL development libraries

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

### Project Structure
```
src/
├── main.rs        # Entry point and CLI handling
├── client.rs      # MySQL client implementation
├── formatter.rs   # Output formatting
└── commands.rs    # Special command handling
```

## 📝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style
- Follow the Rust style guidelines
- Use meaningful variable names
- Add comments for complex logic
- Include tests for new features

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The MySQL team for their amazing database
- The Rust MySQL crate maintainers
- All contributors to this project

## 🔜 Roadmap

- [ ] Tab completion for tables and columns
- [ ] Support for importing/exporting SQL files
- [ ] Better error messages and suggestions
- [ ] Separate main file into components
- [ ] Configuration file support
- [ ] Plugin system for extensions
- [ ] SSH tunnel support
- [ ] Result set pagination
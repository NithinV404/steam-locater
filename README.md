# steam-locater

A terminal user interface (TUI) application built with Rust and Ratatui for listing and managing non-Steam games that have Wine prefixes in your Steam installation.

## Features

- Automatically detects your Steam directory using the `steamlocate` library.
- Lists non-Steam games (shortcuts) that are configured with Wine compatibility tools.
- Interactive navigation: Use arrow keys to select games, Enter to open the Wine prefix directory in your file manager, and 'q' to quit.
- Simple, keyboard-driven interface for quick access to game prefixes.

## Prerequisites

- Rust (install via [rustup](https://rustup.rs/))
- Steam installed on your system (Linux/macOS/Windows)
- Wine (for running Windows games on non-Windows platforms)
- A terminal that supports raw mode (most modern terminals do)

## Installation

1. Clone or download this repository.
2. Navigate to the project directory:
   ```sh
   cd steam-locater
   ```
3. Install the application globally using Cargo:
   ```sh
   cargo install --path .
   ```
   This will build the project in release mode and install the binary to `~/.cargo/bin` (ensure it's in your `PATH`).

## Usage

Run the application from your terminal:
```sh
steam-appid-fetcher
```

### Controls
- **↑/↓**: Navigate through the list of games.
- **Enter**: Open the selected game's Wine prefix directory in your default file manager (using `xdg-open`).
- **q**: Quit the application.

If no non-Steam games with Wine prefixes are found, the application will print a message and exit.

## Dependencies

- [steamlocate](https://crates.io/crates/steamlocate): For locating Steam directories and shortcuts.
- [ratatui](https://crates.io/crates/ratatui): For building the terminal UI.
- [crossterm](https://crates.io/crates/crossterm): For handling terminal input and output.

## Building from Source

If you prefer to build manually:
```sh
cargo build --release
```
The binary will be in `target/release/steam-appid-fetcher`.

## Contributing

Feel free to open issues or submit pull requests for improvements, bug fixes, or new features.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
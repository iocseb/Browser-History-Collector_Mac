# Browser History Exporter

A Rust application that exports browsing history from multiple browsers into a single CSV file. Currently supports Chrome, Firefox, and Safari on macOS.

## Features

- Collects browsing history from multiple browsers:
  - Google Chrome (all user profiles)
  - Mozilla Firefox (all profiles)
  - Safari
- Combines all history entries into a single chronologically sorted CSV file
- Preserves metadata including:
  - Timestamp (in RFC3339 format)
  - URL
  - Page Title
  - Source History File
  - Browser Name
- Creates timestamped output files (e.g., `browser_history_2024-03-21_15-30-45.csv`)
- Handles browser-specific timestamp formats
- Provides helpful error messages for permission issues

## Requirements

- macOS operating system (tested on 14.0.1; Apple Silicon)
- Rust toolchain (1.70.0 or newer)
- Full Disk Access permission for Safari history access

## Installation

1. Clone this repository: 
```bash
git clone https://github.com/yourusername/browser-history-exporter
cd browser-history-exporter
```

2. Build the application:
```bash
cargo build --release
```

3. The executable will be available at `target/release/browser_history_exporter`. You can optionally copy it to a convenient location:
```bash
# Option 1: Copy to system-wide location (requires sudo)
sudo cp target/release/browser_history_exporter /usr/local/bin/browser-history

# Option 2: Copy to your personal bin directory
mkdir -p ~/bin
cp target/release/browser_history_exporter ~/bin/browser-history
```

If you copy to `~/bin`, make sure this directory is in your PATH.

## Usage

Run the application:
```bash
cargo run --release
```

The program will:
1. Search for browser history databases
2. Read available history files
3. Combine all entries
4. Sort them chronologically
5. Export them to a CSV file in the current directory

### Safari Access

To read Safari history, your terminal needs Full Disk Access permission:

1. Open System Settings
2. Navigate to Privacy & Security → Full Disk Access
3. Click the '+' button
4. Add your terminal application (Terminal.app or iTerm)
5. Restart your terminal

## Output Format

The generated CSV file contains the following columns:
- Timestamp: Date and time of the visit in RFC3339 format
- URL: The visited webpage URL
- Title: The webpage title
- History File: Full path to the source history file
- Browser: Name of the browser (Chrome/Firefox/Safari)

## Platform Support

Currently, this tool only supports macOS due to platform-specific browser history file locations. Contributions to add support for other operating systems are welcome!

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

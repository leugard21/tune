# Tune

A terminal-based music player written in Rust.

## Features

- TUI interface using `ratatui`
- Audio playback via `rodio` (MP3, FLAC, WAV, OGG)
- Metadata parsing for Artist and Title
- Persistence: Remembers volume, playback mode, and last played track
- Playback modes: Shuffle and Repeat (One/All)
- Mouse support not required; fully keyboard-driven

## Installation

Ensure you have Rust and Cargo installed.

```bash
cargo install --path .
```

## Usage

Run the application:

```bash
cargo run
```

The player scans the Music directory (`~/Music` or configured path) for supported audio files.

## Controls

| Key      | Action                                |
| -------- | ------------------------------------- |
| k / Up   | Move selection up                     |
| j / Down | Move selection down                   |
| Enter    | Play selected track                   |
| Space    | Pause / Resume                        |
| s        | Stop playback                         |
| + / =    | Increase volume                       |
| -        | Decrease volume                       |
| Left     | Seek backward 5s                      |
| Right    | Seek forward 5s                       |
| z        | Toggle Shuffle                        |
| r        | Cycle Repeat Mode (Off -> All -> One) |
| o        | Cycle Sort Mode                       |
| h        | Toggle Help                           |
| q        | Quit                                  |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

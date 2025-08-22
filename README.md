# LinHT-GUI

Linux radio application with framebuffer display support for embedded radio hardware.

## Quick Start

```bash
# Build
cargo build --release

# Run
cargo run

# Cross-compile for LinHT
cargo build --target aarch64-unknown-linux-gnu --release
```

## Usage

1. Place GNU Radio flowgraphs in `flowgraphs/` directory
2. Run: `cargo run` or `./target/release/linht-gui`
3. Controls:
   - `+` / `UP` - Next flowgraph
   - `-` / `DOWN` - Previous flowgraph  
   - `r` / `ENTER` - Run flowgraph
   - `s` / `ESC` - Stop flowgraph
   - `q` - Quit

## Requirements

- Linux with framebuffer support (`/dev/fb0`)
- GNU Radio with Python support
- 160x128 display

## Dependencies

```toml
[dependencies]
fontdue = "0.8"
memmap2 = "0.9"
libc = "0.2"
evdev = "0.12"
```

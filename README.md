# mad-nes
Just another NES emulator written in Rust. This is not meant to be a full featured emulator, but only as a way to teach myself NES emulation. It can run the most popular NES games just fine, but not guaranteed to run every ROM perfectly.

## Supported mappers:
- NROM
- UNROM
- CNROM
- SxROM
- TxROM

## Building and running the project
Checkout the repo, and then use this command to run your favorite NES games
```
cargo run --release -- [/path/to/game.nes]
```

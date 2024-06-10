# scr2bmp

Takes a .scr file (a raw ZX Spectrum VRAM snapshot, can be output by various emulators) and turns it into a .bmp Windows Bitmap file (4 bits per pixel, 256x192 pixels)

## Build

    cargo build --release

build files are placed in `target/release` as per normal cargo build process.

to make a fully static standalone excutable, compile with musl:

    cargo build --release --target=x86_64-unknown-linux-musl

You might need to install the musl target first, easiest way to do that is with rustup

    rustup target add x86_64-unknown-linux-musl

## usage

    scr2bmp <path to input> <path to output>

There are no other options. It just does the one thing.

## Caveats

Flashing is ignored. You get only the base state of each cell with flashing on.

# ChatGPSP
ChatGPSP is an application for Sony PSP that allows you to chat with ChatGPT using your PSP connected to the internet.

## Rust on PSP
> Rust on psp relies on `rust-psp`, which requires to have a `rustc` set in nightly version.

To setup rust nigthly "directory-wise" you can use:

```bash
rustup override set nightly
```

The project main dependencies are:
- cargo-psp `cargo install cargo-psp`
- rust-psp `cargo install psp`.

To create the EBOOT.PBP file, run:

```bash
cargo psp --release # it is recommended to always build in release mode
```

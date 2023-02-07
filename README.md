# TrySP
## Rust on PSP

> Rust on psp relies on `rust-psp`, that requires to have a `rustc` nightly version installed.

To setup rust nigthly "directory-wise" you can use:

```bash
rustup override set nightly
```

The project main dependencies are:
- cargo-psp `cargo install cargo-psp`
- rust-psp `cargo install psp`.

To create the EBOOT.PBP file, run:

```bash
cargo psp
```

# ChatGPSP
ChatGPSP is an application for Sony PSP that allows you to chat with ChatGPT using your PSP connected to the internet.

## Rust on PSP
> Rust on psp relies on `rust-psp`, which requires to have a `rustc` set in nightly version.

> As of 2024-03-06 the latest version supported is rustc 1.77.0-nightly (`rustup override set nightly-2023-12-22-<arch>`).

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

## Running the application
> The application requires a PSP connected to the internet to work.

> I tested it on a PSP 3004, and the smartphone hotspot (with no password), and it worked fine.
> I was not able to run it successfully on PPSSPP emulator, so real hardware is recommended.

1. Get an API key from OpenAI's GPT-3 API
2. export the API key as an environment variable as `OPENAI_API_KEY`
    ```bash
    export OPENAI_API_KEY=your_api_key
    ```
3. Run `cargo psp --release` to build the application in the root directory of the project
4. Copy the `EBOOT.PBP` file to your PSP's `PSP/GAME/<whatever>/` directory
5. Run the application on your PSP.

Enjoy chatting with ChatGPT on your PSP!

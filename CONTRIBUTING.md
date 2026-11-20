# Contributing to Robo-XCopy

Thanks for your interest. Bug reports, ideas and pull requests are welcome.

## Reporting bugs and asking for features

- Bugs: open an [issue](https://github.com/Skyflash/RoboXCopy/issues/new/choose) with the bug report form.
  The robocopy command and exit code shown in the LOG window help a lot.
- Ideas and questions: use [Discussions](https://github.com/Skyflash/RoboXCopy/discussions).
- Security problems: see [SECURITY.md](SECURITY.md); do not open a public issue.

## Building

Robo-XCopy is a Windows-only Rust program (eframe/egui). You need:

- Rust (stable, MSVC target), for example via `winget install Rustlang.Rustup`
- Visual Studio Build Tools with the C++ workload (the linker and `rc.exe`)

```
cargo run              # debug build
cargo test             # some tests run the real robocopy
cargo build --release  # target\release\RoboXCopy.exe
```

## Pull requests

- Keep each pull request to one change, and describe what it does and why.
- Run `cargo test` before opening it.
- Add a line to the `Unreleased` section of [CHANGELOG.md](CHANGELOG.md).
- Adding a robocopy switch: add a row to the tables in `src/options.rs`; they drive both the command line and
  the COPY BUILDER window.
- Do not reuse the original X-Copy logo or artwork: only the layout idea is borrowed.

By contributing you agree that your work is released under the [MIT License](LICENSE).

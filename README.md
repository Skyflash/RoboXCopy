<img src="docs/icon.svg" alt="Robo-XCopy icon" width="72" align="right">

# Robo-XCopy

[![Downloads](https://img.shields.io/github/downloads/Skyflash/RoboXCopy/total?style=flat-square&label=downloads)](https://github.com/Skyflash/RoboXCopy/releases/latest)

A small, fast Windows GUI for `robocopy`, styled after the classic Amiga **X-Copy**.

Website and how-to: https://roboxcopy.com/

- Single ~5 MB `.exe`, no installer, nothing else to install (the C runtime is built in)
- Pick source and destination, tick the options, hit **START**
- **DRY RUN** previews what would happen (`/L`) without touching anything
- Live per-file progress, a file counter and the Amiga-style "track display"
- Works with any Windows language (output parsing does not depend on the OS locale)
- Optional log file, saved wherever you choose (LOG window: `/LOG` or `/LOG+` with `/TEE`)
- Starts as a normal user; **RUN AS ADMIN** restarts it elevated, and it offers to do so when
  Backup mode or the auditing flag (`/COPYALL`) is used, or when a copy hits "access denied"
- **HELP** gadget in the app and a [how-to page](https://roboxcopy.com/howto.html)

## Options

| Gadget | robocopy switch |
|---|---|
| Subfolders, incl. empty | `/E` |
| Mirror (delete extras) | `/MIR` |
| Move instead of copy | `/MOVE` |
| Selection rules (older, newer, changed, lonely, extra, same, tweaked) | `/XO /XN /XC /XL /XX /IS /IT` |
| List only | `/L` |
| Restartable / Backup mode | `/Z` / `/B` |
| What to copy: flags D A T S O U, presets DAT, DATS, DATSO, ALL | `/COPY:flags`, `/COPYALL` |
| Folder flags | `/DCOPY:flags` |
| Age, size, attribute filters, FAT times, DST, junctions, unbuffered I/O, pause | `/MAXAGE /MINAGE /MAX /MIN /IA /XA /FFT /DST /XJ /J /IPG` |
| Multi-thread | `/MT:n` |
| Retries / wait | `/R:n` `/W:n` |
| File mask, exclude files, exclude dirs, extra args | `*.ext`, `/XF`, `/XD`, raw |

Click **ATTRIBS** or **UPDATE** to open the **COPY BUILDER**: every flag with a description, and the resulting
command line. The exact command line is also printed at the top of the log.

> **Careful with Mirror.** `/MIR` deletes files in the destination that are not in the source.
> Use **DRY RUN** first.

## Build

Requires the Rust toolchain and the Visual Studio C++ Build Tools (MSVC linker).

```powershell
cargo run                # debug
cargo build --release    # target\release\RoboXCopy.exe
cargo test
```

## Themes

Press **THEME** to switch between *Modern Amiga* (blue) and *Classic X-Copy* (black, silver and green).
The choice is remembered in `%APPDATA%\RoboXCopy\theme.txt`.

## Code signing

Releases ship with a SHA-256 and a GitHub build attestation (`gh attestation verify RoboXCopy.exe --repo Skyflash/RoboXCopy`).
Free open source code signing from SignPath Foundation is being set up; until then Windows may show a SmartScreen warning
(*More info*, *Run anyway*). See the [code signing policy](https://roboxcopy.com/signing.html).

## Support

If Robo-XCopy saves you time, you can [buy me a coffee](https://buymeacoffee.com/cristiancastellari).

## License

MIT, (c) 2026 [Cristian Castellari](https://cristiancastellari.it)

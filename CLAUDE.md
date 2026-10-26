# Robo-XCopy: project notes

Context for any session, tool or window that opens this project. Keep it current, and keep
`CHANGELOG.md` updated with every change (the owner asked for that explicitly).

## What it is
Windows-only Rust GUI (eframe/egui 0.37) that drives `robocopy`. The look is modelled on the Amiga
**X-Copy Professional** main screen. Owner: Cristian Castellari. Talk to them in Italian; code,
README, CHANGELOG and the site are in English.

## Links
- Repo: https://github.com/Skyflash/RoboXCopy (public, MIT)
- Site (GitHub Pages, source `docs/`): https://roboxcopy.com/ (dedicated domain bought by the owner on Cloudflare,
  2026-10-26; the old `cristiancastellari.it/RoboXCopy/` is redirected to it by GitHub). See "Site SEO" for the DNS setup.
- Buy Me a Coffee: https://buymeacoffee.com/cristiancastellari (site button, README, `.github/FUNDING.yml`)
- Owner site: https://cristiancastellari.it

## Layout
- `src/main.rs`: app state, drawing of the whole screen (`App::draw`), pop-up windows (FILTERS, LOG, ABOUT).
- `src/ui.rs`: hand-painted widgets on a fixed 640x330 virtual canvas that scales with the window
  (`Canvas`, `button`, `value_box`, `arrow`, `drive_icon`, `grid`).
- `src/theme.rs`: `Theme` struct, the two themes (`MODERN`, `CLASSIC`), `THEMES`, persistence in
  `%APPDATA%\RoboXCopy\theme.txt`. To add a theme: add a `static`, append it to `THEMES`.
- `src/options.rs`: `Options` -> robocopy args. Switch tables (`RULES`, `OTHER`, `VALUES`, `COPY_FLAGS`, `DCOPY_FLAGS`)
  drive both `build_args` and the COPY BUILDER window; to add a switch, add a table row (and keep `example` valid, the
  test `robocopy_accepts_every_builder_switch` runs real robocopy with all of them on).
- `src/builder.rs`: the COPY BUILDER window.
- `src/runner.rs`: background thread, channel messages, output parsing, tests that run real robocopy.
- Icon: edit `assets/icon.txt` only (32x32 grid + palette), then run `powershell -File tools\make-icons.ps1` and commit the
  generated files (`assets/icon.ico`, `docs/favicon.ico`, `docs/icon.svg`, `docs/img/icon-*.png`). `build.rs` (winresource,
  needs the Windows SDK `rc.exe`, present with the VS Build Tools C++ workload) embeds the .ico and the version info;
  `src/icon.rs` draws the window icon from the same txt. The GitHub social preview image can only be set by hand in the
  repo settings: the owner uploaded the app screenshot (`docs/img/social.png`, 1280x640 = `modern.png` scaled and centred on
  the app's navy `#0A1E78`; regenerate it after UI changes). The site pages carry `og:`/`twitter:` tags with absolute URLs
  under `https://roboxcopy.com/`.
- `src/sys.rs`: Win32 helpers via FFI (no extra crates): `is_elevated`, `relaunch_as_admin` (ShellExecuteW "runas"),
  `open_url` (rundll32 url.dll).
- `src/shot.rs`: dev-only screenshot generator (feature `screenshot`).
- `docs/index.html` (home, animated demo), `docs/howto.html` (user guide), `docs/site.css` (shared styles),
  `docs/img/` (generated screenshots). The website pages are in English. `.github/workflows/`: `release.yml` (tag `v*` builds and attaches the exe),
  `pages.yml` (deploys `docs/` when it changes).

## Build and run
Toolchain was installed with winget: Rustup (Rust 1.99.1, MSVC target) and VS 2022 Build Tools (C++ workload).
In a fresh shell add `%USERPROFILE%\.cargo\bin` to PATH.
- `cargo run` / `cargo build` (debug build shows a console window), `cargo test`
- `cargo build --release` -> `target\release\RoboXCopy.exe` (~5 MB, takes about 5 minutes, no console)
- Close any running `RoboXCopy.exe` before rebuilding, or the linker cannot overwrite it.

## Website screenshots
`docs/img/modern.png` and `classic.png` are generated, not hand-captured (an OS screen grab can pick up other
windows). Regenerate after any UI change:
```
cargo build --features screenshot
target\debug\RoboXCopy.exe --screenshot docs\img\modern.png --theme 0
target\debug\RoboXCopy.exe --screenshot docs\img\classic.png --theme 1
target\debug\RoboXCopy.exe --screenshot docs\img\help.png --theme 0 --open help      # also: log, filters, admin
```
`docs/img/` holds modern, classic, help, log, filters, admin, builder (`--open builder`); howto.html uses modern,
builder, filters, log and admin.
Never `Stop-Process -Name RoboXCopy`: the owner may have the app open to test it. Only stop PIDs you started.
Code lives in `src/shot.rs` behind the `screenshot` feature (adds the optional `image` dependency); a normal
`cargo build --release` does not include it. The demo state (paths, 52 files, 60%) is set in `App::start_shot`.
The site's animated demo is plain JS at the bottom of `docs/index.html`; to preview it headless use
`msedge --headless=new --screenshot=out.png --virtual-time-budget=6000 file:///.../docs/index.html`.

## Site SEO
- Domain `roboxcopy.com` now serves the site, so it has its own root: `https://roboxcopy.com/sitemap.xml` is ours.
  There is no `robots.txt` yet (nothing to block; add one with `Sitemap: https://roboxcopy.com/sitemap.xml` if wanted),
  and the sitemap should still be submitted in Google Search Console.
- Each page needs: unique `<title>` (about 50 chars) and description (about 150), canonical, og/twitter tags, JSON-LD.
  Every absolute URL (canonical, og, JSON-LD, sitemap, README, `HOWTO_URL` in `src/main.rs`) uses `https://roboxcopy.com/`.
  The custom domain is the `cname` of the repo Pages settings, and `docs/CNAME` holds it too (not needed with Actions
  deployment, but kept so the repo states it and a switch to branch deployment keeps working).
- Text colour on the blue page must stay `--accent` (`#ffc266`); `--orange` (`#ff8800`) is only 3.05:1 there.
- DNS at Cloudflare: apex `A` 185.199.108-111.153 and `AAAA` 2606:50c0:800{0-3}::153, `www` CNAME `skyflash.github.io`,
  all **DNS only (grey cloud)**. With the orange proxy GitHub cannot issue its certificate (`gh api
  repos/Skyflash/RoboXCopy/pages/health` shows `is_proxied: true`, `is_https_eligible: false`) and "Enforce HTTPS" stays
  disabled. Check with that same call; when `is_https_eligible` is true, enforce with
  `gh api -X PUT repos/Skyflash/RoboXCopy/pages -F https_enforced=true`.

## Design decisions and gotchas
- Static C runtime: `.cargo/config.toml` sets `-C target-feature=+crt-static`. Without it the exe imports `vcruntime140.dll`
  (Visual C++ Redistributable), missing on a clean Windows. Found after 0.1.0 by listing the DLL names inside the exe;
  re-check after dependency or toolchain changes (the exe should list only system DLLs: kernel32, user32, ...).
- robocopy prints progress percentages ending in `\r`, not `\n`: the reader splits on both.
- File lines are recognised by shape (tab separated, numeric size field, we pass `/BYTES`), never by
  text, because robocopy output is localised ("Nuovo file" on Italian Windows).
- Output is in the OEM code page: decoded with `MultiByteToWideChar`.
- A trailing backslash before the closing quote breaks robocopy: paths are normalised (`C:\` -> `C:\.`).
- `/MIR` already implies `/E`; the SUBDIRS field is locked to ALL in mirror mode.
- THREADS = 1 means no `/MT`. Exit codes < 8 are success; the bitmask is described in `describe_exit`.
- Reference the owner uses for building commands: https://ss64.com/nt/robocopy.html. Facts verified against real robocopy
  (Italian Windows, non-admin shell): `/IPG` + `/MT` = exit 16; `/B` and `/COPY:...U` = exit 16 without admin
  ("Manage Auditing" / backup rights); `/COPY:DATSO` and `/DCOPY:E|X|DATEX` work; `/XO` means "skip if the target has the
  same date or newer". `/DCOPY` default is `DA`; the log folder must exist.
- Administrator rights: the exe is `asInvoker` (no manifest elevation) on purpose. A normal user cannot harm Windows
  by mistake, and an elevated process does not see the user's mapped drive letters (use UNC paths). Admin is needed for
  `/B` and `/COPYALL`. Flow: banner badge, prompt on START (`Options::needs_admin`), prompt after an "access denied" run
  (`runner::access_denied` looks for `(0x00000005)`), relaunch keeps `--source`/`--dest` only. The UAC relaunch cannot be
  exercised unattended: the owner tried it by hand on 2026-10-26 and confirmed the app restarts elevated. Elevation
  detection and the rest were covered by automated tests.
- Log file: `Options::log_file` / `log_append` -> `/LOG[+]:path` and `/TEE` (without `/TEE` robocopy prints nothing to
  stdout and the live grids would stay empty).
- Drives A: and B: are not probed at startup on purpose (empty floppy letters can stall).
- Release profile is size-optimised (`opt-level = "z"`, LTO, strip, `panic = "abort"`).
- Not code signed: SmartScreen will warn on first launch.
- Do not reuse the original X-Copy logo or artwork (Cachet); only the layout idea is borrowed.
  `xcopy` is also a Microsoft command name: check the name before any rebrand.
- Tooling gotcha: in this environment a Bash heredoc halves backslashes; use Rust raw strings
  (`r"C:\a"`) in tests and the Edit/Write tools for files containing Windows paths.
- Screenshots of the window need a DPI-aware capture, and the window must be in front: other windows
  can overlap it. Do not keep screenshots that include other applications.

## Roadmap
See `CHANGELOG.md` > Unreleased > Planned. Agreed order: publish v0.1, then real progress / profiles / log,
then evaluate the disk TOOLKIT.

TOOLKIT constraints (raw disk clone / wipe): needs admin (relaunch elevated only for that mode), must lock
and dismount volumes, must refuse the system disk, show model/serial/size and require typed confirmation;
zero-fill does not securely erase SSDs (use Secure Erase / NVMe format); reading real Amiga floppies needs
hardware such as Greaseweazle or KryoFlux. The track grids map naturally onto disk slices.

## History
On 2026-10-26, with the owner's explicit go-ahead, the 15 development commits were replaced by ONE initial commit
(author: Cristian Castellari, GitHub no-reply email `6575896+Skyflash@users.noreply.github.com`), force-pushed with
`--force-with-lease`, and `v0.1.0` was tagged on it. A bundle of the old history was kept OUTSIDE the repo at
`RoboXCopy-history-before-squash.bundle`, next to the project folder on the owner's PC (restore with `git clone <bundle> dir`). From here on
work normally with small commits; do not rewrite the public history again without asking.
The old commits remain reachable by SHA on GitHub for a while and in any fork; nothing sensitive was in them.

## Release procedure
1. Update `CHANGELOG.md` (move items from Unreleased into a dated version section: the release notes are taken from it)
   and the version in `Cargo.toml`; commit and push to `main`.
2. Optional dry run without publishing: `gh workflow run release.yml --ref main` (tests, build, checksum, artifact).
3. `git tag -a vX.Y.Z -m "Robo-XCopy X.Y.Z" && git push origin vX.Y.Z`: the workflow tests, builds, writes the SHA-256,
   attests the build provenance and publishes `RoboXCopy.exe` + `RoboXCopy.exe.sha256` with the CHANGELOG notes.
4. Verify without inflating the download counter: download the run artifact (`gh run download <id> -n RoboXCopy`), not
   the release asset, and run `gh attestation verify RoboXCopy.exe --repo Skyflash/RoboXCopy`.

# Changelog

All notable changes to Robo-XCopy are recorded here.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org/).

## [Unreleased]

### Planned
- Installer (Inno Setup, per-user, with the portable exe kept) and a winget package.
- Code signing (SignPath Foundation for open source, or Trusted Signing); until then SHA-256 and build attestation.
- True overall progress: a `/L` pre-scan for total files/bytes, overall bar and ETA.
- Profiles: save and load source, target and options (including the log file choice).
- Notification when a copy finishes.
- Drive sound effect during copy (the original X-Copy was audible).
- Bitmap Topaz-style font and a metal-plate logo for the Classic theme.
- Drag & drop folders onto the window; Italian and English UI.
- TOOLKIT mode (separate phase): raw disk clone, wipe and verify, with bad-sector handling.
  Needs admin rights, volume locking, system-disk protection and typed confirmation.
  See `CLAUDE.md` for the constraints.

## [0.1.0] - 2026-10-26

First public version.

### The program
- GUI front end for Windows `robocopy`, laid out like the Amiga X-Copy Professional screen: banner, parameter row with
  up/down arrows, gadget row, drive icons with light bulbs, two 10x8 track grids, status bar.
- Fields MODE (copy / mirror / move), SUBDIRS, THREADS, RETRY, WAIT, ATTRIBS, UPDATE. ATTRIBS and UPDATE summarise the
  COPY BUILDER (`DAT`, `ALL`, `NEWER`, `3 RULES`).
- Gadgets START, REPEAT, STOP, DRY RUN (`/L`), FILTERS, LOG, THEME, HELP.
- **COPY BUILDER** (click ATTRIBS or UPDATE, or the button in FILTERS): graphical, described choice of the `/COPY` flags
  (D A T S O U; presets DAT / DATS / DATSO / ALL = `/COPYALL`; `/COPY:DAT` is passed explicitly), `/DCOPY` flags
  (D A T E X), selection rules (`/XO /XN /XC /XL /XX /IS /IT`), other switches (`/PURGE /FFT /DST /XJ /J`), value switches
  (`/MAXAGE /MINAGE /MAX /MIN /IA /XA /IPG`) and a live preview of the command. Driven by tables in `src/options.rs`.
- FILTERS window: restartable `/Z`, backup `/B`, list only `/L`, file mask, `/XF`, `/XD`, raw extra args.
- Track grids: the left one lights a cell per file processed, the right one shows the current file's progress.
- **Log file** chosen from the GUI (LOG window: CHOOSE / NONE, Append, Unicode `/UNILOG` for accented and non-Latin
  names). Adds `/LOG[+]:file` and `/TEE`, so the live output keeps working. The log folder is checked before START.
- **Administrator rights**: starts as a normal user and shows `RUN AS ADMIN` (or `* ADMIN *`) in the banner. A prompt
  offers RESTART AS ADMIN / CONTINUE ANYWAY / CANCEL when START is pressed with Backup mode or the auditing flag
  (`U`, part of `/COPYALL`), which make robocopy refuse to start, and after a run that hit "access denied"
  (error 5, found by its language-independent code `(0x00000005)`) or exit code 16. The restart keeps source and target.
- Two themes selectable with THEME and remembered in `%APPDATA%\RoboXCopy\theme.txt`: *Modern Amiga* (blue, default)
  and *Classic X-Copy* (black, silver frames, green values).
- HELP window with a quick start and links to the how-to and the project page.
- Icon: a pixel-art floppy disk with an orange X, drawn once in `assets/icon.txt`; the exe icon, version info
  (product name, description, copyright), window icon, favicon and site images are all generated from it.
- Works on any Windows language: output parsing does not depend on the OS locale. OEM code page decoding, so accented
  file names display correctly. Safe handling of paths (trailing backslash and drive-root quirks of robocopy).
- Single exe of about 4.7 MB, no installer and no runtime.

### Found by running real robocopy in the tests
- `/IPG` is refused together with `/MT` (exit 16, nothing copied), so a pause switches the copy to one thread.
- `/B` and the `U` flag make robocopy refuse to start without administrator rights; security `S` and owner `O` work
  on your own files. That is what the admin prompt is based on.
- `/XO` skips a file when the target's copy has the same date or is newer.

### Website and project
- Site at https://roboxcopy.com/ (GitHub Pages, `docs/`): animated demo window, screenshots of both themes, a full
  how-to page, download counter (appears once there is a download), Buy Me a Coffee link.
- SEO: titles and descriptions, canonical, schema.org JSON-LD, Open Graph and Twitter tags, sitemap, favicon, a
  1280x640 social preview image, text contrast that meets WCAG AA.
- 20 automated tests, including ones that run the real robocopy with every switch the builder can produce.
- Dev-only `screenshot` cargo feature (not in the release exe) that renders the app and saves the frame as a PNG.
- MIT license, (c) 2026 Cristian Castellari.

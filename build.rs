// Embeds the icon and the version info in RoboXCopy.exe (Explorer, taskbar, Task Manager).
// The icon is generated from assets/icon.txt by tools/make-icons.ps1.
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Robo-XCopy");
        res.set("FileDescription", "Robo-XCopy - a GUI for robocopy");
        res.set("LegalCopyright", "(c) 2026 Cristian Castellari");
        res.set("OriginalFilename", "RoboXCopy.exe");
        if let Err(e) = res.compile() {
            // A missing resource compiler must not stop the build: the app just gets no icon in Explorer.
            println!("cargo:warning=could not embed the icon: {e}");
        }
    }
}

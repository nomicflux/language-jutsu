import AppKit

@main enum InstallerMain {
    static func main() {
        let app = NSApplication.shared
        let delegate = InstallerApp()
        app.delegate = delegate
        app.setActivationPolicy(.regular)
        app.run()
    }
}

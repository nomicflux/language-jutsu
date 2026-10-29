// Test-only entry point. It uses the production window and installer with an isolated home.
import AppKit

@main enum InstallerUITestMain {
    static func main() throws {
        let path = try String(contentsOf: Bundle.main.resourceURL!.appendingPathComponent("test-home.txt"), encoding: .utf8)
        let app = NSApplication.shared
        let delegate = InstallerApp(paths: InstallationPaths(home: URL(fileURLWithPath: path)))
        app.delegate = delegate
        app.setActivationPolicy(.regular)
        app.run()
    }
}

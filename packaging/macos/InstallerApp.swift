import AppKit
import Foundation

final class InstallerApp: NSObject, NSApplicationDelegate, NSWindowDelegate {
    private var window: NSWindow!
    private let message = NSTextField(wrappingLabelWithString: "Install Language Jutsu and its listening skill for your Mac. Guided lessons use Codex or Claude Code.")
    private let status = NSTextField(wrappingLabelWithString: "The speech model downloads automatically (about 1.5 GiB).")
    private let spinner = NSProgressIndicator()
    private let install = NSButton(title: "Install", target: nil, action: nil)
    private let cancel = NSButton(title: "Cancel", target: nil, action: nil)
    private let lessons = NSButton(title: "Open Lessons Folder", target: nil, action: nil)
    private let codex = NSButton(title: "Copy Codex Prompt", target: nil, action: nil)
    private let claude = NSButton(title: "Copy Claude Code Prompt", target: nil, action: nil)
    private let details = NSTextView()
    private let detailsLabel = NSTextField(labelWithString: "Details")
    private let detailsScroll = NSScrollView()
    private var worker: Installer?
    private var running = false
    private var log: URL?
    private let paths: InstallationPaths

    init(paths: InstallationPaths = InstallationPaths(home: FileManager.default.homeDirectoryForCurrentUser)) {
        self.paths = paths
        super.init()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 330),
                          styleMask: [.titled, .closable, .miniaturizable], backing: .buffered, defer: false)
        window.title = "Install Language Jutsu"
        window.delegate = self
        let title = NSTextField(labelWithString: "Language Jutsu")
        title.font = .boldSystemFont(ofSize: 24)
        spinner.style = .spinning
        spinner.controlSize = .small
        spinner.isDisplayedWhenStopped = false
        let progress = NSStackView(views: [spinner, status])
        progress.orientation = .horizontal
        let links = NSStackView(views: [link("Get Codex", "https://developers.openai.com/codex/app/"),
                                       link("Get Claude Code", "https://code.claude.com/docs/en/desktop-quickstart")])
        links.orientation = .horizontal
        install.target = self; install.action = #selector(start)
        cancel.target = self; cancel.action = #selector(cancelInstall); cancel.isHidden = true
        lessons.target = self; lessons.action = #selector(openLessons); lessons.isHidden = true
        codex.target = self; codex.action = #selector(copyCodex); codex.isHidden = true
        claude.target = self; claude.action = #selector(copyClaude); claude.isHidden = true
        let actions = NSStackView(views: [install, cancel, lessons])
        actions.orientation = .horizontal
        let prompts = NSStackView(views: [codex, claude])
        prompts.orientation = .horizontal
        let scroll = detailsScroll
        scroll.hasVerticalScroller = true
        scroll.borderType = .bezelBorder
        details.isEditable = false
        details.font = .monospacedSystemFont(ofSize: 11, weight: .regular)
        details.isVerticallyResizable = true
        details.autoresizingMask = [.width]
        scroll.documentView = details
        scroll.heightAnchor.constraint(equalToConstant: 135).isActive = true
        detailsLabel.isHidden = true; scroll.isHidden = true
        let stack = NSStackView(views: [title, message, links, progress, actions, prompts,
                                        detailsLabel, scroll])
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 13
        stack.translatesAutoresizingMaskIntoConstraints = false
        window.contentView!.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: window.contentView!.leadingAnchor, constant: 24),
            stack.trailingAnchor.constraint(equalTo: window.contentView!.trailingAnchor, constant: -24),
            stack.topAnchor.constraint(equalTo: window.contentView!.topAnchor, constant: 24),
            scroll.widthAnchor.constraint(equalTo: stack.widthAnchor),
            message.widthAnchor.constraint(equalTo: stack.widthAnchor),
            status.widthAnchor.constraint(lessThanOrEqualTo: stack.widthAnchor, constant: -30)
        ])
        window.center(); window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    private func link(_ title: String, _ url: String) -> NSButton {
        let button = NSButton(title: title, target: self, action: #selector(openLink(_:)))
        button.identifier = NSUserInterfaceItemIdentifier(url)
        return button
    }

    @objc private func openLink(_ sender: NSButton) {
        if let value = sender.identifier?.rawValue, let url = URL(string: value) { NSWorkspace.shared.open(url) }
    }

    @objc private func start() {
        guard !running, let resources = Bundle.main.resourceURL else { return }
        // A universal app normally runs natively. sysctl also handles an Intel launch under Rosetta.
        var arm64: Int32 = 0
        var size = MemoryLayout<Int32>.size
        sysctlbyname("hw.optional.arm64", &arm64, &size, nil, 0)
        let installer = Installer(resources: resources, paths: paths, architecture: arm64 == 1 ? "arm64" : "x86_64")
        let conflicts = installer.conflicts()
        if !conflicts.isEmpty {
            let alert = NSAlert()
            alert.messageText = "Replace the existing listening skill?"
            alert.informativeText = "The existing skill will be backed up beside its current location. Saved lessons will not be moved or deleted.\n\n" + conflicts.map(\.path).joined(separator: "\n")
            alert.addButton(withTitle: "Replace Skill"); alert.addButton(withTitle: "Cancel")
            guard alert.runModal() == .alertFirstButtonReturn else { return }
        }
        let logURL = FileManager.default.temporaryDirectory.appendingPathComponent("language-jutsu-setup-\(UUID().uuidString).log")
        log = logURL; worker = installer; running = true
        install.isEnabled = false; cancel.isHidden = false
        detailsLabel.isHidden = true; detailsScroll.isHidden = true
        spinner.startAnimation(nil); details.string = ""
        DispatchQueue.global(qos: .userInitiated).async {
            do {
                try installer.install(replacing: Set(conflicts.map(\.path)), log: logURL) { text in
                    DispatchQueue.main.async { self.status.stringValue = text }
                }
                DispatchQueue.main.async { self.finish(error: nil) }
            } catch {
                DispatchQueue.main.async { self.finish(error: error) }
            }
        }
    }

    private func finish(error: Error?) {
        running = false; worker = nil
        spinner.stopAnimation(nil); cancel.isHidden = true
        if let log = log { details.string = (try? String(contentsOf: log, encoding: .utf8)) ?? "" }
        if let error = error {
            status.stringValue = error.localizedDescription
            detailsLabel.isHidden = false; detailsScroll.isHidden = false
            window.setContentSize(NSSize(width: 600, height: 490))
            install.title = "Retry"; install.isEnabled = true
        } else {
            install.isHidden = true; lessons.isHidden = false; codex.isHidden = false; claude.isHidden = false
            message.stringValue = "Open Codex or Claude Code and paste its prompt to start a lesson. Restart your assistant if the skill does not appear. You can now eject the installer disk."
        }
    }

    @objc private func cancelInstall() { worker?.cancellation.cancel() }
    @objc private func openLessons() { NSWorkspace.shared.open(paths.lessons) }
    private func copy(_ text: String) { NSPasteboard.general.clearContents(); NSPasteboard.general.setString(text, forType: .string) }
    @objc private func copyCodex() { copy("$language-jutsu-listening Help me choose and prepare my first listening lesson.") }
    @objc private func copyClaude() { copy("/language-jutsu-listening Help me choose and prepare my first listening lesson.") }
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
    func windowShouldClose(_ sender: NSWindow) -> Bool {
        applicationShouldTerminate(NSApp) == .terminateNow
    }
    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        if running {
            let alert = NSAlert()
            alert.messageText = "Setup is still running"
            alert.informativeText = "Use Cancel and wait for setup to stop before closing the installer."
            alert.runModal()
            return .terminateCancel
        }
        return .terminateNow
    }
}

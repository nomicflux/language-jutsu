import Foundation

struct InstallationPaths {
    let home: URL
    var root: URL { home.appendingPathComponent("Library/Application Support/Language Jutsu") }
    var lessons: URL { home.appendingPathComponent("Documents/Language Jutsu/listening") }
    var skill: URL { root.appendingPathComponent("skills/language-jutsu-listening") }
    var skillLinks: [URL] {
        [".agents/skills/language-jutsu-listening", ".claude/skills/language-jutsu-listening"]
            .map { home.appendingPathComponent($0) }
    }
}

enum InstallError: LocalizedError {
    case message(String)
    case cancelled
    var errorDescription: String? {
        switch self {
        case .message(let text): return text
        case .cancelled: return "Setup was cancelled. Click Retry to try again."
        }
    }
}

final class InstallCancellation {
    private let lock = NSLock()
    private var cancelled = false
    private var process: Process?

    func cancel() {
        lock.lock(); defer { lock.unlock() }
        cancelled = true
        if let process = process, process.isRunning { process.terminate() }
    }

    func check() throws {
        lock.lock(); defer { lock.unlock() }
        if cancelled { throw InstallError.cancelled }
    }

    func launch(_ child: Process) throws {
        lock.lock(); defer { lock.unlock() }
        if cancelled { throw InstallError.cancelled }
        try child.run()
        process = child
    }

    func finished() {
        lock.lock(); defer { lock.unlock() }
        process = nil
    }
}

final class Installer {
    static let marker = "language-jutsu-macos-v1\n"
    let resources: URL
    let paths: InstallationPaths
    let architecture: String
    let cancellation: InstallCancellation
    let environment: [String: String]
    private let fm = FileManager.default

    init(resources: URL, paths: InstallationPaths, architecture: String,
         cancellation: InstallCancellation = InstallCancellation(),
         environment: [String: String] = ProcessInfo.processInfo.environment) {
        self.resources = resources
        self.paths = paths
        self.architecture = architecture
        self.cancellation = cancellation
        self.environment = environment
    }

    private func exists(_ url: URL) -> Bool {
        (try? fm.attributesOfItem(atPath: url.path)) != nil
    }

    func conflicts() -> [URL] {
        paths.skillLinks.filter { link in
            guard exists(link) else { return false }
            guard let target = try? fm.destinationOfSymbolicLink(atPath: link.path) else { return true }
            let absolute = URL(fileURLWithPath: target, relativeTo: link.deletingLastPathComponent()).standardizedFileURL
            return absolute.path != paths.skill.standardizedFileURL.path
        }
    }

    private func run(_ executable: URL, _ arguments: [String], in directory: URL, log: URL, failure: String) throws {
        try cancellation.check()
        if !fm.fileExists(atPath: log.path) { fm.createFile(atPath: log.path, contents: nil) }
        let output = try FileHandle(forWritingTo: log)
        defer { try? output.close() }
        try output.seekToEnd()
        try output.write(contentsOf: Data(("\n> " + executable.lastPathComponent + " " + arguments.joined(separator: " ") + "\n").utf8))
        let process = Process()
        process.executableURL = executable
        process.arguments = arguments
        process.currentDirectoryURL = directory
        process.environment = environment
        process.standardInput = FileHandle.nullDevice
        process.standardOutput = output
        process.standardError = output
        try cancellation.launch(process)
        process.waitUntilExit()
        cancellation.finished()
        try cancellation.check()
        guard process.terminationReason == .exit && process.terminationStatus == 0 else {
            throw InstallError.message(failure + " See Details below, then click Retry.")
        }
    }

    func install(replacing approved: Set<String>, log: URL, status: (String) -> Void) throws {
        guard ["arm64", "x86_64"].contains(architecture) else {
            throw InstallError.message("This installer supports Apple Silicon and Intel Macs.")
        }
        let unexpected = Set(conflicts().map(\.path)).subtracting(approved)
        guard unexpected.isEmpty else {
            throw InstallError.message("An existing skill needs your permission to replace it.")
        }
        if exists(paths.root) {
            guard !((try? fm.attributesOfItem(atPath: paths.root.path)[.type]) as? FileAttributeType == .typeSymbolicLink),
                  (try? String(contentsOf: paths.root.appendingPathComponent(".language-jutsu-installation"), encoding: .utf8)) == Self.marker else {
                throw InstallError.message("The installation folder already contains files from another source: \(paths.root.path). Nothing was replaced.")
            }
        }
        let parent = paths.root.deletingLastPathComponent()
        try fm.createDirectory(at: parent, withIntermediateDirectories: true)
        let stage = parent.appendingPathComponent(".Language-Jutsu-\(UUID().uuidString)")
        let backup = parent.appendingPathComponent(".Language-Jutsu-previous-\(UUID().uuidString)")
        defer { try? fm.removeItem(at: stage) }
        status("Installing Language Jutsu…")
        try fm.copyItem(at: resources.appendingPathComponent("common"), to: stage)
        let payload = resources.appendingPathComponent("payloads/\(architecture)")
        for name in ["runtime", "licenses"] {
            try fm.copyItem(at: payload.appendingPathComponent(name), to: stage.appendingPathComponent(name))
        }
        let cli = stage.appendingPathComponent("bin/language-jutsu")
        status("Downloading speech model… About 1.5 GiB on the first installation.")
        try run(cli, ["download-model"], in: stage, log: log, failure: "The speech model could not be downloaded or opened.")
        status("Checking audio processing and transcription…")
        let sample = resources.appendingPathComponent("sample.wav").path
        let check = stage.appendingPathComponent("setup-check")
        try fm.createDirectory(at: check, withIntermediateDirectories: false)
        try run(cli, [sample, "--language", "en", "--start", "0", "--end", "4",
                      "--output", check.appendingPathComponent("study.mp3").path], in: check, log: log, failure: "The audio processing check failed.")
        try run(cli, [sample, "--language", "en", "--start", "0", "--end", "4", "--transcribe",
                      "--output", check.appendingPathComponent("transcript.txt").path], in: check, log: log, failure: "The speech recognition check failed.")
        try fm.removeItem(at: check)
        try cancellation.check()
        // Do not publish a partially prepared installation or remove the previous one on failure.
        status("Connecting the listening skill…")
        guard Set(conflicts().map(\.path)).subtracting(approved).isEmpty else {
            throw InstallError.message("A skill installation changed during setup. Click Retry to review it.")
        }
        var replacedRoot = false
        var publishedRoot = false
        var installedLinks: [URL] = []
        var savedLinks: [(URL, URL)] = []
        do {
            try fm.createDirectory(at: paths.lessons, withIntermediateDirectories: true)
            if exists(paths.root) {
                try fm.moveItem(at: paths.root, to: backup)
                replacedRoot = true
            }
            try fm.moveItem(at: stage, to: paths.root)
            publishedRoot = true
            for link in paths.skillLinks {
                try fm.createDirectory(at: link.deletingLastPathComponent(), withIntermediateDirectories: true)
                if exists(link) {
                    // Preserve different installations. Keep their backup after a successful install.
                    let saved = link.deletingLastPathComponent().appendingPathComponent(".language-jutsu-listening-previous-\(UUID().uuidString)")
                    try fm.moveItem(at: link, to: saved)
                    savedLinks.append((link, saved))
                }
                try fm.createSymbolicLink(at: link, withDestinationURL: paths.skill)
                installedLinks.append(link)
            }
        } catch {
            for link in installedLinks { try? fm.removeItem(at: link) }
            for (link, saved) in savedLinks { try? fm.moveItem(at: saved, to: link) }
            if publishedRoot { try? fm.removeItem(at: paths.root) }
            if replacedRoot { try? fm.moveItem(at: backup, to: paths.root) }
            throw error
        }
        if replacedRoot { try? fm.removeItem(at: backup) }
        for (link, saved) in savedLinks where !approved.contains(link.path) { try? fm.removeItem(at: saved) }
        status("Language Jutsu is ready.")
    }
}

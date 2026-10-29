import Foundation

@main enum InstallerTests {
    static func require(_ condition: @autoclosure () throws -> Bool, _ message: String) throws {
        if try !condition() { throw InstallError.message(message) }
    }

    static func main() throws {
        let fm = FileManager.default
        let temporary = fm.temporaryDirectory.appendingPathComponent("Jutsu installer tests \(UUID().uuidString)")
        defer { try? fm.removeItem(at: temporary) }
        let resources = temporary.appendingPathComponent("resources")
        let home = temporary.appendingPathComponent("Learner's Home")
        let paths = InstallationPaths(home: home)
        let common = resources.appendingPathComponent("common")
        let runtime = resources.appendingPathComponent("payloads/arm64/runtime")
        for directory in [common.appendingPathComponent("bin"), common.appendingPathComponent("skills/language-jutsu-listening"),
                          runtime, resources.appendingPathComponent("payloads/arm64/licenses")] {
            try fm.createDirectory(at: directory, withIntermediateDirectories: true)
        }
        try Installer.marker.write(to: common.appendingPathComponent(".language-jutsu-installation"), atomically: true, encoding: .utf8)
        try "skill".write(to: common.appendingPathComponent("skills/language-jutsu-listening/SKILL.md"), atomically: true, encoding: .utf8)
        let skillScripts = common.appendingPathComponent("skills/language-jutsu-listening/scripts")
        try fm.createDirectory(at: skillScripts, withIntermediateDirectories: false)
        try fm.copyItem(at: URL(fileURLWithPath: "skills/language-jutsu-listening/scripts/jutsu"), to: skillScripts.appendingPathComponent("jutsu"))
        let launcher = common.appendingPathComponent("bin/language-jutsu")
        try fm.copyItem(at: URL(fileURLWithPath: "packaging/macos/launcher.sh"), to: launcher)
        try fm.setAttributes([.posixPermissions: 0o755], ofItemAtPath: launcher.path)
        let fake = runtime.appendingPathComponent("language-jutsu")
        let script = """
        #!/bin/sh
        set -eu
        if [ "$1" = download-model ]; then
            if [ -f "$FAIL_FILE" ]; then echo 'network unavailable' >&2; exit 1; fi
            if [ -f "$SLEEP_FILE" ]; then exec /bin/sleep 30; fi
            echo 'Model cached: /a model with spaces'
            exit 0
        fi
        while [ "$#" -gt 0 ]; do
            if [ "$1" = --output ]; then shift; printf 'sample' > "$1"; exit 0; fi
            shift
        done
        exit 2
        """
        try script.write(to: fake, atomically: true, encoding: .utf8)
        try fm.setAttributes([.posixPermissions: 0o755], ofItemAtPath: fake.path)
        let failFile = temporary.appendingPathComponent("fail")
        let sleepFile = temporary.appendingPathComponent("sleep")
        let environment = ["FAIL_FILE": failFile.path, "SLEEP_FILE": sleepFile.path]
        let installer = Installer(resources: resources, paths: paths, architecture: "arm64", environment: environment)
        let log = temporary.appendingPathComponent("setup.log")
        try installer.install(replacing: [], log: log) { _ in }
        try require(fm.fileExists(atPath: paths.root.appendingPathComponent("runtime/language-jutsu").path), "Runtime was not installed")
        for link in paths.skillLinks {
            let destination = try fm.destinationOfSymbolicLink(atPath: link.path)
            try require(destination == paths.skill.path, "Skill points outside installed payload")
            let process = Process()
            process.executableURL = link.appendingPathComponent("scripts/jutsu")
            process.arguments = ["download-model"]
            process.currentDirectoryURL = temporary
            process.environment = environment
            try process.run()
            process.waitUntilExit()
            try require(process.terminationStatus == 0, "Skill runner failed through its installed link")
        }
        let lesson = paths.lessons.appendingPathComponent("keep.txt")
        try "learner work".write(to: lesson, atomically: true, encoding: .utf8)
        try installer.install(replacing: [], log: log) { _ in }
        try require(try String(contentsOf: lesson) == "learner work", "Update changed lesson data")

        try "fail".write(to: failFile, atomically: true, encoding: .utf8)
        do {
            try installer.install(replacing: [], log: log) { _ in }
            throw InstallError.message("Expected model failure")
        } catch InstallError.message(let text) where text.contains("speech model could not") { }
        try require(fm.fileExists(atPath: paths.skill.appendingPathComponent("SKILL.md").path), "Failed update destroyed installed skill")
        try fm.removeItem(at: failFile)
        try installer.install(replacing: [], log: log) { _ in }

        let conflict = paths.skillLinks[0]
        try fm.removeItem(at: conflict)
        try fm.createDirectory(at: conflict, withIntermediateDirectories: true)
        try "old skill".write(to: conflict.appendingPathComponent("keep.txt"), atomically: true, encoding: .utf8)
        try require(installer.conflicts().map(\.path) == [conflict.path], "Wrong conflicts: \(installer.conflicts().map(\.path))")
        do {
            try installer.install(replacing: [], log: log) { _ in }
            throw InstallError.message("Replaced a skill without consent")
        } catch InstallError.message(let text) where text.contains("permission") { }
        try installer.install(replacing: [conflict.path], log: log) { _ in }
        let backups = try fm.contentsOfDirectory(at: conflict.deletingLastPathComponent(), includingPropertiesForKeys: nil)
            .filter { $0.lastPathComponent.hasPrefix(".language-jutsu-listening-previous-") }
        try require(backups.count == 1, "Conflicting skill was not backed up")
        try require(try String(contentsOf: backups[0].appendingPathComponent("keep.txt")) == "old skill", "Backup lost old skill")

        try "sleep".write(to: sleepFile, atomically: true, encoding: .utf8)
        let cancellation = InstallCancellation()
        let cancellable = Installer(resources: resources, paths: paths, architecture: "arm64", cancellation: cancellation, environment: environment)
        DispatchQueue.global().asyncAfter(deadline: .now() + 0.3) { cancellation.cancel() }
        do {
            try cancellable.install(replacing: [], log: log) { _ in }
            throw InstallError.message("Cancellation did not stop setup")
        } catch InstallError.cancelled { }
        try fm.removeItem(at: sleepFile)
        try installer.install(replacing: [], log: log) { _ in }
        try require(try String(contentsOf: lesson) == "learner work", "Cancellation changed saved lesson")
        // Force a failure after the new runtime and first skill link have been published.
        let sentinel = paths.root.appendingPathComponent("old-runtime.txt")
        try "keep runtime".write(to: sentinel, atomically: true, encoding: .utf8)
        let claudeSkills = paths.skillLinks[1].deletingLastPathComponent()
        let savedClaude = home.appendingPathComponent("saved-claude-skills")
        try fm.moveItem(at: claudeSkills, to: savedClaude)
        try "unrelated file".write(to: claudeSkills, atomically: true, encoding: .utf8)
        var failedPublication = false
        do { try installer.install(replacing: [], log: log) { _ in } }
        catch { failedPublication = true }
        try require(failedPublication, "Expected blocked skill-directory failure")
        try require(try String(contentsOf: sentinel) == "keep runtime", "Publication failure lost previous runtime")
        try require(try String(contentsOf: claudeSkills) == "unrelated file", "Publication changed unrelated file")
        try require(try fm.destinationOfSymbolicLink(atPath: paths.skillLinks[0].path) == paths.skill.path,
                    "Publication failure did not restore the first skill link")
        try fm.removeItem(at: claudeSkills)
        try fm.moveItem(at: savedClaude, to: claudeSkills)
        try installer.install(replacing: [], log: log) { _ in }
        print("Installer tests passed: install, update, failure/retry, conflict/backup, cancellation, publication rollback, paths with spaces.")
    }
}

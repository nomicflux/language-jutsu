"""Release orchestration against real temporary Git repositories and a simulated GitHub."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]

BUILD = '''import os, pathlib, sys, tomllib
root = pathlib.Path.cwd()
if os.environ.get("FAIL_BUILD"): sys.exit(1)
if os.environ.get("CHANGE_SOURCE"): (root / "src/app.rs").write_text("changed while building")
work = root / "target/macos"
if sys.argv[1] == "payload":
    (work / "payloads" / sys.argv[3]).mkdir(parents=True, exist_ok=True)
    (work / "sources").mkdir(exist_ok=True)
    (work / "sources/source.txt").write_text("dependency source")
else:
    assert all((work / "payloads" / arch).exists() for arch in ("arm64", "x86_64"))
    version = tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"]
    (work / f"Language-Jutsu-{version}-unsigned.dmg").write_text(version)
'''

GH = '''#!/usr/bin/env python3
import json, os, pathlib, subprocess, sys
path = pathlib.Path(os.environ["RELEASE_STATE"])
state = json.loads(path.read_text())
args = sys.argv[1:]
state["calls"].append(args)
def save(): path.write_text(json.dumps(state))
save()
if args[:2] == ["auth", "status"]:
    sys.exit(0 if state["authenticated"] else 1)
if args[:2] == ["auth", "login"]:
    state["authenticated"] = True
elif args[:2] == ["release", "list"]:
    assert args[args.index("--json") + 1] == "tagName,isDraft"
    print(json.dumps(state["releases"]))
else:
    assert args[args.index("--repo") + 1] == "nomicflux/language-jutsu"
    action, tag = args[1:3]
    if action == "create":
        assert "--draft" in args
        state["releases"].append({"tagName": tag, "isDraft": True})
    elif action == "upload":
        if os.environ.get("FAIL_UPLOAD"): sys.exit(1)
        files = [pathlib.Path(name) for name in args[3:6]]
        assert len(files) == 3 and all(file.is_file() for file in files)
        assert files[0].read_text() == tag[1:]
    elif action == "edit" and "--draft=false" in args:
        assert "--latest" in args
        for release in state["releases"]:
            if release["tagName"] == tag: release["isDraft"] = False
        subprocess.run(["git", "tag", tag], check=True)
        subprocess.run(["git", "push", "origin", tag], check=True)
save()
'''


class ReleaseTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="Jutsu release test ")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name).resolve()
        self.repo = self.directory / "checkout with spaces"
        self.repo.mkdir()
        files = {
            ".gitignore": "/target/\n", "Cargo.toml": '[package]\nname = "language-jutsu"\nversion = "0.1.0"\n[dependencies]\ndemo = "0.1.0"\n',
            "Cargo.lock": '[[package]]\nname = "language-jutsu"\nversion = "0.1.0"\n[[package]]\nname = "demo"\nversion = "0.1.0"\n',
            "README.md": "# App\nInstall and use it.\n[Download Language Jutsu for Mac](https://github.com/nomicflux/language-jutsu/releases/download/v0.1.0/old.dmg)\n",
            "src/app.rs": "old source", "src/obsolete.rs": "old file", "formats/default.json": "{}",
            "skills/language-jutsu-listening/SKILL.md": "old skill", "tests/example.rs": "test",
            "setup.sh": "setup", "setup.ps1": "setup", ".github/workflows/build.yml": "workflow",
            "packaging/macos/OPENING.txt": "Opening instructions", "packaging/macos/build.py": BUILD,
        }
        for name in ("scripts/package-macos.sh", "scripts/package_macos.py"):
            files[name] = (ROOT / name).read_text()
        for name, text in files.items():
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        (self.repo / "scripts/package-macos.sh").chmod(0o755)
        self.git("init", "-b", "main")
        self.git("config", "user.name", "Release Test")
        self.git("config", "user.email", "test@example.invalid")
        self.git("add", ".")
        self.git("commit", "-m", "Initial")
        self.git("init", "--bare", str(self.directory / "origin.git"))
        self.git("remote", "add", "origin", str(self.directory / "origin.git"))
        self.git("tag", "v0.1.0")
        self.git("push", "-u", "origin", "main", "--tags")
        self.initial = self.git("rev-parse", "HEAD")
        self.state = self.directory / "releases.json"
        self.state.write_text(json.dumps({"authenticated": True, "calls": [], "releases": [{"tagName": "v0.1.0", "isDraft": False}]}))
        tools = self.directory / "tools"
        tools.mkdir()
        for name in ("gh", "rustup", "cargo", "cmake", "swiftc", "hdiutil"):
            path = tools / name
            path.write_text(GH if name == "gh" else "#!/bin/sh\nexit 0\n")
            path.chmod(0o755)
        self.env = dict(os.environ, PATH=f"{tools}:{os.environ['PATH']}", RELEASE_STATE=str(self.state))
        self.env.pop("SWIFT_EXEC", None)

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.repo, text=True, capture_output=True, check=True).stdout.strip()

    def release(self, *args, success=True, **environment):
        result = subprocess.run([str(self.repo / "scripts/package-macos.sh"), *args], cwd=self.directory,
                                env=dict(self.env, **environment), text=True, capture_output=True)
        self.assertEqual(result.returncode == 0, success, result.stdout + result.stderr)
        return result

    def assert_version(self, version):
        manifest = tomllib.loads((self.repo / "Cargo.toml").read_text())
        locked = tomllib.loads((self.repo / "Cargo.lock").read_text())["package"]
        self.assertEqual(manifest["package"]["version"], version)
        self.assertEqual(locked[0]["version"], version)
        self.assertEqual(locked[1]["version"], "0.1.0")
        self.assertEqual(manifest["dependencies"]["demo"], "0.1.0")

    def test_one_command_includes_app_changes_but_preserves_personal_files_and_staging(self):
        (self.repo / "src/app.rs").write_text("new source")
        (self.repo / "src/obsolete.rs").unlink()
        (self.repo / "formats/new.json").write_text("{}")
        (self.repo / "skills/language-jutsu-listening/SKILL.md").write_text("new skill")
        (self.repo / "listening").mkdir()
        (self.repo / "listening/private.txt").write_text("private lesson")
        (self.repo / "notes.txt").write_text("unrelated staged work")
        self.git("add", "notes.txt")
        self.release()
        self.assert_version("0.1.1")
        self.assertEqual(self.git("show", "HEAD:src/app.rs"), "new source")
        self.assertEqual(self.git("show", "HEAD:skills/language-jutsu-listening/SKILL.md"), "new skill")
        self.assertNotIn("src/obsolete.rs", self.git("ls-tree", "-r", "--name-only", "HEAD"))
        self.assertEqual(self.git("diff", "--cached", "--name-only"), "notes.txt")
        self.assertIn("?? listening/", self.git("status", "--short"))
        self.assertIn("/releases/latest)", (self.repo / "README.md").read_text())
        self.assertEqual(self.git("rev-parse", "HEAD"), self.git("rev-parse", "origin/main"))
        self.assertEqual(self.git("rev-parse", "HEAD"), self.git("rev-parse", "v0.1.1"))

    def test_explicit_version(self):
        self.release("0.2.0")
        self.assert_version("0.2.0")

    def test_subsequent_run_picks_next_patch(self):
        self.release()
        self.release()
        self.assert_version("0.1.2")

    def test_existing_and_invalid_versions_do_not_mutate_files(self):
        for version in ("0.1.0", "bad-version"):
            self.release(version, success=False)
            self.assertEqual(self.git("status", "--porcelain"), "")

    def test_build_failure_can_be_retried_without_version_instructions(self):
        self.release(success=False, FAIL_BUILD="1")
        self.assertEqual(self.git("rev-parse", "HEAD"), self.initial)
        self.release()
        self.assert_version("0.1.1")

    def test_upload_failure_resumes_draft_without_an_extra_commit(self):
        self.release(success=False, FAIL_UPLOAD="1")
        commit = self.git("rev-parse", "HEAD")
        self.release()
        self.assertEqual(self.git("rev-parse", "HEAD"), commit)
        self.assert_version("0.1.1")
        state = json.loads(self.state.read_text())
        self.assertEqual(sum(call[:2] == ["release", "create"] for call in state["calls"]), 1)
        self.assertFalse(state["releases"][-1]["isDraft"])

    def test_changed_source_stops_before_committing_or_publishing(self):
        self.release(success=False, CHANGE_SOURCE="1")
        self.assertEqual(self.git("rev-parse", "HEAD"), self.initial)
        self.assertEqual(len(json.loads(self.state.read_text())["releases"]), 1)

    def test_login_is_started_when_needed(self):
        state = json.loads(self.state.read_text())
        state["authenticated"] = False
        self.state.write_text(json.dumps(state))
        self.release()
        self.assertIn(["auth", "login"], json.loads(self.state.read_text())["calls"])


if __name__ == "__main__":
    unittest.main()

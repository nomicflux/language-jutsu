# Install Language Jutsu's tools, Whisper model and listening skill (Windows on x64).
$ErrorActionPreference = 'Stop'
if ($env:PROCESSOR_ARCHITECTURE -ne 'AMD64') { throw 'setup.ps1 supports Windows on x64' }
$repo = $PSScriptRoot
$skill = Join-Path $repo 'skills\language-jutsu-listening'
Set-Location $repo

function Update-Path {
    $env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' +
        [Environment]::GetEnvironmentVariable('Path', 'User') + ";$env:USERPROFILE\.cargo\bin"
}

function Install-Package($id) {
    winget install --source winget --exact --id $id --accept-source-agreements --accept-package-agreements
    if ($LASTEXITCODE -ne 0) { throw "winget install $id failed" }
}

function Install-Rust {
    $installer = Join-Path $env:TEMP 'rustup-init.exe'
    Invoke-WebRequest 'https://win.rustup.rs/x86_64' -OutFile $installer
    & $installer
}

function Install-Skill($skillsDir) {
    $destination = Join-Path $skillsDir 'language-jutsu-listening'
    if (-not (Test-Path $destination)) {
        New-Item -ItemType Directory -Force $skillsDir | Out-Null
        New-Item -ItemType Junction -Path $destination -Target $skill | Out-Null
        Write-Output "Installed: $destination -> $skill"
    } elseif ((Get-Item $destination).Target -eq $skill) {
        Write-Output "Already installed: $destination"
    } else {
        throw "Cannot install: $destination already exists. Nothing was replaced."
    }
}

function Invoke-Cargo {
    cargo @args
    if ($LASTEXITCODE -ne 0) { throw "cargo $args failed" }
}

# yt-dlp's winget package also installs Deno and FFmpeg (with ffprobe).
if (-not (Get-Command yt-dlp -ErrorAction SilentlyContinue)) { Install-Package yt-dlp.yt-dlp }
if (-not (Get-Command cmake -ErrorAction SilentlyContinue)) { Install-Package Kitware.CMake }
if (-not (Test-Path "$env:ProgramFiles\LLVM\bin\libclang.dll")) { Install-Package LLVM.LLVM }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Install-Rust }
Update-Path
Invoke-Cargo build
Write-Output 'Downloading the Whisper model (about 1.5 GiB on first run)...'
Invoke-Cargo run --quiet -- download-model
Install-Skill (Join-Path $env:USERPROFILE '.agents\skills')
Install-Skill (Join-Path $env:USERPROFILE '.claude\skills')
Write-Output 'Setup complete.'

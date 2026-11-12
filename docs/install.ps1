# TURBOCURSA installer for Windows (PowerShell).
#   irm https://turbocursa.chiuta.ro/install.ps1 | iex
# Downloads the latest release to %LOCALAPPDATA%\TURBOCURSA and adds a Start Menu shortcut.
$ErrorActionPreference = "Stop"
$repo = "chiuta/TurboCursa"
$dir = Join-Path $env:LOCALAPPDATA "TURBOCURSA"
New-Item -ItemType Directory -Force $dir | Out-Null
$zip = Join-Path $env:TEMP "turbocursa.zip"
$url = "https://github.com/$repo/releases/latest/download/turbocursa-windows-x86_64.zip"
try {
  Invoke-WebRequest $url -OutFile $zip -UseBasicParsing
  Expand-Archive $zip -DestinationPath $dir -Force
  Remove-Item $zip -Force
} catch {
  if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "No release binary yet; building from source (a few minutes)..."
    cargo install --git "https://github.com/$repo" --features native,audio --bin turbo --root $dir --force
    Move-Item (Join-Path $dir "bin\turbo.exe") (Join-Path $dir "turbocursa.exe") -Force
  } else {
    Write-Host "No release binary yet and cargo is not installed. Install Rust from https://rustup.rs and run this again."
    exit 1
  }
}
$exe = Join-Path $dir "turbocursa.exe"
$lnk = Join-Path ([Environment]::GetFolderPath("Programs")) "TURBOCURSA.lnk"
$s = (New-Object -ComObject WScript.Shell).CreateShortcut($lnk)
$s.TargetPath = $exe; $s.WorkingDirectory = $dir; $s.Save()
Write-Host "Installed: $exe (Start Menu: TURBOCURSA)"

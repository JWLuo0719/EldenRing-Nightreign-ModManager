$ErrorActionPreference = "Stop"
$Host.UI.RawUI.WindowTitle = "Nightreign Mod Manager"

$ProjectRoot = $PSScriptRoot
$AppDir = Join-Path $ProjectRoot "app"

function Test-Project {
    if (-not (Test-Path (Join-Path $AppDir "package.json"))) {
        throw "Cannot find app\package.json. Run this script from the project root."
    }

    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        throw "npm is not available in PATH."
    }

    if (-not (Get-Command npx -ErrorAction SilentlyContinue)) {
        throw "npx is not available in PATH."
    }
}

function Invoke-AppCommand {
    param(
        [string]$Title,
        [scriptblock]$Command
    )

    Write-Host ""
    Write-Host "==> $Title" -ForegroundColor Yellow
    Push-Location $AppDir
    try {
        & $Command
    }
    finally {
        Pop-Location
    }
}

function Pause-Menu {
    Write-Host ""
    Read-Host "Press Enter to continue"
}

Test-Project

while ($true) {
    Clear-Host
    Write-Host ""
    Write-Host "  ========================================"
    Write-Host "      Nightreign Mod Manager"
    Write-Host "  ========================================"
    Write-Host ""
    Write-Host "  [1] Tauri Dev (Full App)"
    Write-Host "  [2] Vite Dev (Frontend Only)"
    Write-Host "  [3] Build Frontend"
    Write-Host "  [4] Tauri Production Build"
    Write-Host "  [5] Lint Check"
    Write-Host "  [6] Rust Check"
    Write-Host "  [7] Rust Test"
    Write-Host "  [0] Exit"
    Write-Host ""
    Write-Host "  ========================================"
    Write-Host ""

    $choice = Read-Host "  Select [0-7]"

    try {
        switch ($choice) {
            "1" {
                Invoke-AppCommand "Tauri Dev" { npx tauri dev }
                Pause-Menu
            }
            "2" {
                Invoke-AppCommand "Vite Dev" { npm run dev }
                Pause-Menu
            }
            "3" {
                Invoke-AppCommand "Frontend Build" { npm run build }
                Pause-Menu
            }
            "4" {
                Invoke-AppCommand "Tauri Production Build" { npx tauri build }
                Pause-Menu
            }
            "5" {
                Invoke-AppCommand "Lint Check" { npm run lint }
                Pause-Menu
            }
            "6" {
                Invoke-AppCommand "Rust Check" {
                    Push-Location "src-tauri"
                    try {
                        cargo check
                    }
                    finally {
                        Pop-Location
                    }
                }
                Pause-Menu
            }
            "7" {
                Invoke-AppCommand "Rust Test" {
                    Push-Location "src-tauri"
                    try {
                        cargo test
                    }
                    finally {
                        Pop-Location
                    }
                }
                Pause-Menu
            }
            "0" {
                exit 0
            }
            default {
                Write-Host "Invalid selection." -ForegroundColor Red
                Pause-Menu
            }
        }
    }
    catch {
        Write-Host ""
        Write-Host "[ERROR] $($_.Exception.Message)" -ForegroundColor Red
        Pause-Menu
    }
}

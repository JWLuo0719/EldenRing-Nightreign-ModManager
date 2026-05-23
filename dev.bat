@echo off
setlocal

set "ROOT_DIR=%~dp0"
set "APP_DIR=%ROOT_DIR%app"

if not exist "%APP_DIR%\package.json" (
  echo [ERROR] Cannot find app\package.json.
  echo Run this script from the project root.
  pause
  exit /b 1
)

where npm >nul 2>nul
if errorlevel 1 (
  echo [ERROR] npm is not available in PATH.
  pause
  exit /b 1
)

where npx >nul 2>nul
if errorlevel 1 (
  echo [ERROR] npx is not available in PATH.
  pause
  exit /b 1
)

cd /d "%APP_DIR%" || (
  echo [ERROR] Failed to enter app directory.
  pause
  exit /b 1
)

echo Starting Nightreign Mod Manager with Tauri dev...
echo Tauri will start the Vite dev server through beforeDevCommand.
echo.

npx tauri dev
set "EXIT_CODE=%ERRORLEVEL%"

echo.
echo Tauri dev exited with code %EXIT_CODE%.
pause
exit /b %EXIT_CODE%

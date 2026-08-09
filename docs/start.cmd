@echo off
rem Double-click this instead of index.html.
rem A wasm module is fetched, and fetch() refuses the file:// scheme, so opening
rem index.html directly can never work - it has to come over HTTP.
setlocal
cd /d "%~dp0"
set PORT=8124

powershell -NoProfile -Command "try { Invoke-WebRequest -Uri 'http://localhost:%PORT%/' -UseBasicParsing -TimeoutSec 2 | Out-Null; exit 0 } catch { exit 1 }"
if errorlevel 1 (
  echo starting local server on port %PORT% ...
  start "ownership-server" /min powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0serve.ps1" -Port %PORT%
  powershell -NoProfile -Command "Start-Sleep -Seconds 2"
) else (
  echo server already running on port %PORT%
)

start "" "http://localhost:%PORT%/"
endlocal

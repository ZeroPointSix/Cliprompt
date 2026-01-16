@echo off
setlocal
set "RUST_BIN=E:\hushaokang\Software\Rust\cargo\bin"
set "PATH=%RUST_BIN%;%PATH%"
set "PROJECT_ROOT=D:\Z-note\胡大的二级笔记\项目\Prompnt lanucher\prompt-launcher"
set "LOG_PATH=%PROJECT_ROOT%\dev-watch.log"

echo %date% %time% watch start>>"%LOG_PATH%"

:loop
echo %date% %time% start tauri dev>>"%LOG_PATH%"
cd /d "%PROJECT_ROOT%"
call npm run tauri dev>>"%LOG_PATH%" 2>&1
echo %date% %time% tauri dev exited, retry in 30s>>"%LOG_PATH%"
timeout /t 30 /nobreak>nul
goto loop

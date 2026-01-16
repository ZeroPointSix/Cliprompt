$ErrorActionPreference = "Stop"

$projectRoot = "D:\Z-note\胡大的二级笔记\项目\Prompnt lanucher\prompt-launcher"
$logPath = Join-Path $projectRoot "dev-watch.log"

$env:CARGO_HOME = "E:\hushaokang\Software\Rust\cargo"
$env:RUSTUP_HOME = "E:\hushaokang\Software\Rust\rustup"
$env:Path = "E:\hushaokang\Software\Rust\cargo\bin;$env:Path"

function Write-Log([string]$message) {
  $stamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
  "$stamp $message" | Out-File -FilePath $logPath -Append -Encoding UTF8
}

Write-Log "watch start"

while ($true) {
  Write-Log "start tauri dev"
  $arguments = "/c npm run tauri dev >> `"$logPath`" 2>&1"
  $process = Start-Process -FilePath "cmd.exe" -ArgumentList $arguments -WorkingDirectory $projectRoot -PassThru -NoNewWindow
  $process.WaitForExit()
  Write-Log "tauri dev exited with code $($process.ExitCode), retry in 30s"
  Start-Sleep -Seconds 30
}

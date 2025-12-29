# Kill flow-paste dev processes
$processes = @("flow-paste", "flow-paste.exe")
foreach ($proc in $processes) {
    Get-Process -Name $proc -ErrorAction SilentlyContinue | Stop-Process -Force
}
Write-Host "Dev processes cleaned up." -ForegroundColor Green

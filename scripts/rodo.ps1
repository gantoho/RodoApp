param (
    [switch]$Clean,
    [switch]$Release
)

# 标题和时间
$startTime = Get-Date
Write-Host "=== Rodo 构建脚本 ===" -ForegroundColor Cyan
Write-Host "开始时间: $startTime" -ForegroundColor Gray
Write-Host ""

# 如果指定了清理选项
if ($Clean) {
    Write-Host "🧹 清理项目..." -ForegroundColor Yellow
    cargo clean
    Write-Host ""
}

# 根据是否是发布版本选择构建命令
$buildCmd = if ($Release) { "build --release" } else { "build" }

# 构建应用
Write-Host "🔨 构建应用..." -ForegroundColor Cyan
Invoke-Expression "cargo $buildCmd"

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 构建失败!" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""

# 运行应用
Write-Host "🚀 运行应用..." -ForegroundColor Green
$runCmd = if ($Release) { "run --release" } else { "run" }
Invoke-Expression "cargo $runCmd"

# 计算运行时间
$endTime = Get-Date
$duration = $endTime - $startTime
Write-Host ""
Write-Host "✅ 完成!" -ForegroundColor Yellow
Write-Host "总耗时: $($duration.Minutes)分 $($duration.Seconds)秒" -ForegroundColor Gray 
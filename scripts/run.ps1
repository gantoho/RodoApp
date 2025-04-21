#!/usr/bin/env pwsh
param (
    [switch]$Clean,
    [switch]$Release,
    [switch]$Help
)

# 显示帮助信息
function Show-Help {
    Write-Host "RODO 构建运行脚本" -ForegroundColor Cyan
    Write-Host "用法: ./run.ps1 [选项]" -ForegroundColor White
    Write-Host ""
    Write-Host "选项:"
    Write-Host "  -Clean      在构建前清理项目" -ForegroundColor Yellow
    Write-Host "  -Release    使用发布模式构建和运行" -ForegroundColor Yellow
    Write-Host "  -Help       显示此帮助信息" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "示例:"
    Write-Host "  ./run.ps1                # 标准构建和运行" -ForegroundColor DarkGray
    Write-Host "  ./run.ps1 -Clean         # 清理后构建和运行" -ForegroundColor DarkGray
    Write-Host "  ./run.ps1 -Release       # 发布模式构建和运行" -ForegroundColor DarkGray
    Write-Host "  ./run.ps1 -Clean -Release # 清理后以发布模式构建和运行" -ForegroundColor DarkGray
    exit 0
}

# 如果请求帮助，显示帮助信息并退出
if ($Help) {
    Show-Help
}

# 标题和时间
$startTime = Get-Date
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "          RODO 构建运行脚本           " -ForegroundColor White
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "开始时间: $($startTime.ToString('HH:mm:ss'))" -ForegroundColor DarkGray
Write-Host ""

# 清理项目（如果指定了-Clean参数）
if ($Clean) {
    Write-Host "🧹 清理项目..." -ForegroundColor Yellow
    cargo clean
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ 清理失败!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host "✓ 清理完成" -ForegroundColor Green
    Write-Host ""
}

# 根据是否指定了-Release参数，确定构建模式
$buildMode = if ($Release) { "发布" } else { "调试" }
$buildCmd = if ($Release) { "build --release" } else { "build" }

# 构建应用
Write-Host "🔨 使用${buildMode}模式构建..." -ForegroundColor Cyan
Invoke-Expression "cargo $buildCmd"
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 构建失败!" -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "✓ 构建完成" -ForegroundColor Green
Write-Host ""

# 运行应用
$runMode = if ($Release) { "发布" } else { "调试" }
$runCmd = if ($Release) { "run --release" } else { "run" }
Write-Host "🚀 运行${runMode}版本..." -ForegroundColor Magenta
Invoke-Expression "cargo $runCmd"

# 计算运行时间
$endTime = Get-Date
$duration = $endTime - $startTime
$durationStr = if ($duration.Hours -gt 0) {
    "{0}小时 {1}分 {2}秒" -f $duration.Hours, $duration.Minutes, $duration.Seconds
} else {
    "{0}分 {1}秒" -f $duration.Minutes, $duration.Seconds
}

Write-Host ""
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "✅ 完成!" -ForegroundColor Green
Write-Host "总运行时间: $durationStr" -ForegroundColor DarkGray
Write-Host "=======================================" -ForegroundColor Cyan 
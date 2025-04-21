#!/usr/bin/env pwsh
param (
    [switch]$Clean,
    [switch]$Help
)

# 显示帮助信息
function Show-Help {
    Write-Host "Rodo Windows打包脚本" -ForegroundColor Cyan
    Write-Host "用法: ./windows_build.ps1 [选项]" -ForegroundColor White
    Write-Host ""
    Write-Host "选项:"
    Write-Host "  -Clean      在构建前清理项目" -ForegroundColor Yellow
    Write-Host "  -Help       显示此帮助信息" -ForegroundColor Yellow
    Write-Host ""
    exit 0
}

# 如果请求帮助，显示帮助信息并退出
if ($Help) {
    Show-Help
}

# 标题和时间
$startTime = Get-Date
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "       Rodo Windows打包脚本            " -ForegroundColor White
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "开始时间: $($startTime.ToString('HH:mm:ss'))" -ForegroundColor DarkGray
Write-Host ""

# 确保cargo-bundle安装
Write-Host "🔧 检查cargo-bundle..." -ForegroundColor Yellow
$hasBundler = cargo install --list | Select-String -Pattern "cargo-bundle"
if (-not $hasBundler) {
    Write-Host "安装cargo-bundle（用于打包Windows应用）..." -ForegroundColor Yellow
    cargo install cargo-bundle
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ 安装cargo-bundle失败!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

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

# 确保目录结构
Write-Host "📁 确保目录结构..." -ForegroundColor Yellow
$dirs = @(
    "assets\fonts",
    "assets\icons",
    "target\release\bundle"
)

foreach ($dir in $dirs) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Host "  创建目录: $dir" -ForegroundColor Gray
    }
}
Write-Host "✓ 目录检查完成" -ForegroundColor Green
Write-Host ""

# 构建发布版本
Write-Host "🔨 构建发布版本..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 构建失败!" -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "✓ 构建完成" -ForegroundColor Green
Write-Host ""

# 创建Windows打包
Write-Host "📦 创建Windows打包..." -ForegroundColor Cyan
cargo bundle --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 打包失败!" -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "✓ 打包完成" -ForegroundColor Green
Write-Host ""

# 打包为ZIP
Write-Host "🗜️ 创建ZIP分发包..." -ForegroundColor Yellow
$version = (Get-Content "Cargo.toml" | Select-String -Pattern 'version = "(.*)"').Matches.Groups[1].Value
$zipFile = "target\Rodo-$version-windows.zip"

if (Test-Path $zipFile) {
    Remove-Item $zipFile -Force
}

Compress-Archive -Path "target\release\bundle\windows\*" -DestinationPath $zipFile
Write-Host "✓ ZIP包创建成功: $zipFile" -ForegroundColor Green
Write-Host ""

# 计算运行时间
$endTime = Get-Date
$duration = $endTime - $startTime
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "✅ 打包完成!" -ForegroundColor Green
Write-Host "总耗时: $($duration.Minutes)分 $($duration.Seconds)秒" -ForegroundColor DarkGray
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "可以在以下位置找到打包文件:" -ForegroundColor White
Write-Host "- Windows安装包: target\release\bundle\windows\" -ForegroundColor White
Write-Host "- ZIP分发包: $zipFile" -ForegroundColor White
Write-Host "" 
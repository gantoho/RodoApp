#!/usr/bin/env pwsh
# 下载所需的Noto Sans SC字体文件

# 标题显示
Write-Host "=================================" -ForegroundColor Cyan
Write-Host "      Rodo字体下载脚本           " -ForegroundColor White
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

# 确保目录存在
$fontDir = "assets/fonts"
if (-not (Test-Path $fontDir)) {
    Write-Host "创建字体目录: $fontDir" -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $fontDir -Force | Out-Null
}

# 设置下载URL和目标文件路径
$fontUrl = "https://github.com/kartotherian/osm-bright.fonts/raw/master/fonts/NotoSansSC-Regular.otf"
$fontPath = "$fontDir/NotoSansSC-Regular.otf"

# 检查是否已存在字体文件
if (Test-Path $fontPath) {
    Write-Host "字体文件已存在: $fontPath" -ForegroundColor Yellow
    $response = Read-Host "是否要重新下载? (y/N)"
    if ($response -ne "y" -and $response -ne "Y") {
        Write-Host "下载已取消，使用现有字体文件。" -ForegroundColor Green
        exit 0
    }
}

# 下载字体文件
Write-Host "正在下载Noto Sans SC Regular字体..." -ForegroundColor Cyan
try {
    Invoke-WebRequest -Uri $fontUrl -OutFile $fontPath
    Write-Host "✓ 字体下载成功!" -ForegroundColor Green
    Write-Host "字体已保存到: $fontPath" -ForegroundColor White
} catch {
    Write-Host "❌ 字体下载失败:" -ForegroundColor Red
    Write-Host $_.Exception.Message
    Write-Host ""
    Write-Host "请手动下载字体文件并放置在以下位置:" -ForegroundColor Yellow
    Write-Host $fontPath -ForegroundColor White
    Write-Host "可从此处下载: https://fonts.google.com/noto/specimen/Noto+Sans+SC" -ForegroundColor White
    exit 1
}

Write-Host ""
Write-Host "=================================" -ForegroundColor Cyan
Write-Host "           完成               " -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Cyan 
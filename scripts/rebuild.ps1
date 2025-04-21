Write-Host "=== 清理项目 ===" -ForegroundColor Cyan
cargo clean
Write-Host ""

Write-Host "=== 重新构建 ===" -ForegroundColor Cyan
cargo build
Write-Host ""

Write-Host "=== 运行应用 ===" -ForegroundColor Green
cargo run
Write-Host ""

Write-Host "=== 完成 ===" -ForegroundColor Yellow 
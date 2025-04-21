@echo off
echo === 清理项目 ===
cargo clean
echo.

echo === 重新构建 ===
cargo build
echo.

echo === 运行应用 ===
cargo run
echo.

echo === 完成 === 
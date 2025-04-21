@echo off
setlocal enabledelayedexpansion

:: 获取开始时间
set start=%time%

:: 显示标题
echo ===================================
echo       RODO 构建运行脚本
echo ===================================
echo.

:: 参数处理
set clean=0
set release=0

:parse
if "%~1"=="" goto endparse
if /i "%~1"=="-c" set clean=1
if /i "%~1"=="--clean" set clean=1
if /i "%~1"=="-r" set release=1
if /i "%~1"=="--release" set release=1
if /i "%~1"=="-h" goto help
if /i "%~1"=="--help" goto help
shift
goto parse
:endparse

:: 如果指定了清理选项
if %clean%==1 (
    echo [清理项目...]
    cargo clean
    if errorlevel 1 (
        echo 清理失败！
        exit /b 1
    )
    echo 清理完成。
    echo.
)

:: 根据是否是发布版本选择构建命令
if %release%==1 (
    echo [使用发布模式构建...]
    cargo build --release
) else (
    echo [使用调试模式构建...]
    cargo build
)

:: 检查构建状态
if errorlevel 1 (
    echo 构建失败！
    exit /b 1
)
echo 构建完成。
echo.

:: 运行应用
if %release%==1 (
    echo [运行发布版本...]
    cargo run --release
) else (
    echo [运行调试版本...]
    cargo run
)

:: 计算运行时间
set end=%time%
set options="tokens=1-4 delims=:.,"
for /f %options% %%a in ("%start%") do set start_h=%%a&set /a start_m=100%%b %% 100&set /a start_s=100%%c %% 100
for /f %options% %%a in ("%end%") do set end_h=%%a&set /a end_m=100%%b %% 100&set /a end_s=100%%c %% 100

set /a hours=%end_h%-%start_h%
set /a mins=%end_m%-%start_m%
set /a secs=%end_s%-%start_s%
if %hours% lss 0 set /a hours = 24%hours%
if %mins% lss 0 set /a hours = %hours% - 1 & set /a mins = 60%mins%
if %secs% lss 0 set /a mins = %mins% - 1 & set /a secs = 60%secs%

:: 显示总运行时间
echo.
echo ===================================
echo 完成！
if %hours% gtr 0 echo 总运行时间: %hours%小时 %mins%分 %secs%秒
if %hours% equ 0 echo 总运行时间: %mins%分 %secs%秒
echo ===================================
exit /b 0

:help
echo.
echo RODO 构建运行脚本帮助：
echo 用法: rebuild.cmd [选项]
echo.
echo 选项:
echo   -c, --clean    在构建前清理项目
echo   -r, --release  使用发布模式构建
echo   -h, --help     显示此帮助信息
echo.
echo 示例:
echo   rebuild.cmd            - 标准构建和运行
echo   rebuild.cmd -c         - 清理后构建和运行
echo   rebuild.cmd -r         - 发布模式构建和运行
echo   rebuild.cmd -c -r      - 清理后以发布模式构建和运行
exit /b 0 
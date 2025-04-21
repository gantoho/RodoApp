#!/bin/bash

# Rodo Android构建脚本
# 用于构建Android版本的Rodo应用

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # 无颜色

# 打印帮助信息
show_help() {
    echo -e "${CYAN}Rodo Android打包脚本${NC}"
    echo "用法: ./android_build.sh [选项]"
    echo ""
    echo "选项:"
    echo -e "  -c, --clean      ${YELLOW}在构建前清理项目${NC}"
    echo -e "  -h, --help       ${YELLOW}显示此帮助信息${NC}"
    echo ""
    exit 0
}

# 解析命令行参数
CLEAN=0

while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--clean)
            CLEAN=1
            shift
            ;;
        -h|--help)
            show_help
            ;;
        *)
            echo -e "${RED}错误: 未知选项 $1${NC}"
            show_help
            ;;
    esac
done

# 显示标题
echo -e "${CYAN}=========================================${NC}"
echo -e "${CYAN}         Rodo Android打包脚本           ${NC}"
echo -e "${CYAN}=========================================${NC}"
echo "开始时间: $(date +%H:%M:%S)"
echo ""

# 检查必要的工具
echo -e "${YELLOW}🔍 检查必要工具...${NC}"

# 检查Rust工具链
if ! command -v rustup &> /dev/null; then
    echo -e "${RED}❌ 未找到rustup! 请先安装Rust工具链${NC}"
    exit 1
fi

# 检查Android NDK
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo -e "${YELLOW}⚠️ ANDROID_NDK_HOME环境变量未设置${NC}"
    echo -e "${YELLOW}请确保已安装Android NDK并设置环境变量${NC}"
fi

# 检查cargo-apk
if ! cargo install --list | grep -q "cargo-apk"; then
    echo -e "${YELLOW}安装cargo-apk（用于Android应用构建）...${NC}"
    cargo install cargo-apk
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ 安装cargo-apk失败!${NC}"
        exit 1
    fi
fi

# 添加Android目标
echo -e "${YELLOW}确保安装Android目标...${NC}"
rustup target add aarch64-linux-android armv7-linux-androideabi

# 清理项目（如果指定了--clean选项）
if [ $CLEAN -eq 1 ]; then
    echo -e "${YELLOW}🧹 清理项目...${NC}"
    cargo clean
    echo -e "${GREEN}✓ 清理完成${NC}"
    echo ""
fi

# 确保目录结构
echo -e "${YELLOW}📁 确保目录结构...${NC}"
mkdir -p assets/fonts assets/icons target/android

# 构建Android APK
echo -e "${CYAN}🔨 构建Android APK...${NC}"
cargo apk build --release

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ 构建失败!${NC}"
    exit 1
fi

echo -e "${GREEN}✓ 构建完成${NC}"
echo ""

# 复制APK到输出目录
echo -e "${YELLOW}📦 复制APK到输出目录...${NC}"
VERSION=$(grep 'version = ' Cargo.toml | head -1 | cut -d '"' -f 2)
APK_FILE="target/android-artifacts/release/apk/rodo.apk"
OUTPUT_FILE="target/Rodo-${VERSION}-android.apk"

if [ -f "$APK_FILE" ]; then
    cp "$APK_FILE" "$OUTPUT_FILE"
    echo -e "${GREEN}✓ APK已复制到: ${OUTPUT_FILE}${NC}"
else
    echo -e "${RED}❌ 找不到构建的APK文件!${NC}"
    exit 1
fi

# 结束计时
echo ""
echo -e "${CYAN}=========================================${NC}"
echo -e "${GREEN}✅ 打包完成!${NC}"
echo "结束时间: $(date +%H:%M:%S)"
echo -e "${CYAN}=========================================${NC}"
echo -e "可以在以下位置找到打包文件:"
echo -e "- Android APK: ${OUTPUT_FILE}"
echo ""

echo -e "${YELLOW}提示: 要在设备上安装APK，可以运行:${NC}"
echo -e "  adb install -r ${OUTPUT_FILE}"
echo "" 
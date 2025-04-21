#!/bin/bash
# 下载所需的Noto Sans SC字体文件

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m' # 无颜色

# 标题显示
echo -e "${CYAN}=================================${NC}"
echo -e "${CYAN}      Rodo字体下载脚本           ${NC}"
echo -e "${CYAN}=================================${NC}"
echo ""

# 确保目录存在
FONT_DIR="assets/fonts"
if [ ! -d "$FONT_DIR" ]; then
    echo -e "${YELLOW}创建字体目录: $FONT_DIR${NC}"
    mkdir -p "$FONT_DIR"
fi

# 设置下载URL和目标文件路径
FONT_URL="https://github.com/kartotherian/osm-bright.fonts/raw/master/fonts/NotoSansSC-Regular.otf"
FONT_PATH="$FONT_DIR/NotoSansSC-Regular.otf"

# 检查是否已存在字体文件
if [ -f "$FONT_PATH" ]; then
    echo -e "${YELLOW}字体文件已存在: $FONT_PATH${NC}"
    read -p "是否要重新下载? (y/N) " response
    if [[ ! "$response" =~ ^[yY]$ ]]; then
        echo -e "${GREEN}下载已取消，使用现有字体文件。${NC}"
        exit 0
    fi
fi

# 下载字体文件
echo -e "${CYAN}正在下载Noto Sans SC Regular字体...${NC}"

if command -v curl &> /dev/null; then
    # 使用curl下载
    if curl -L -o "$FONT_PATH" "$FONT_URL"; then
        echo -e "${GREEN}✓ 字体下载成功!${NC}"
        echo -e "字体已保存到: $FONT_PATH"
    else
        echo -e "${RED}❌ 字体下载失败${NC}"
        echo ""
        echo -e "${YELLOW}请手动下载字体文件并放置在以下位置:${NC}"
        echo "$FONT_PATH"
        echo "可从此处下载: https://fonts.google.com/noto/specimen/Noto+Sans+SC"
        exit 1
    fi
elif command -v wget &> /dev/null; then
    # 使用wget下载
    if wget -O "$FONT_PATH" "$FONT_URL"; then
        echo -e "${GREEN}✓ 字体下载成功!${NC}"
        echo -e "字体已保存到: $FONT_PATH"
    else
        echo -e "${RED}❌ 字体下载失败${NC}"
        echo ""
        echo -e "${YELLOW}请手动下载字体文件并放置在以下位置:${NC}"
        echo "$FONT_PATH"
        echo "可从此处下载: https://fonts.google.com/noto/specimen/Noto+Sans+SC"
        exit 1
    fi
else
    echo -e "${RED}❌ 无法下载字体: 系统未安装curl或wget${NC}"
    echo ""
    echo -e "${YELLOW}请手动下载字体文件并放置在以下位置:${NC}"
    echo "$FONT_PATH"
    echo "可从此处下载: https://fonts.google.com/noto/specimen/Noto+Sans+SC"
    exit 1
fi

echo ""
echo -e "${CYAN}=================================${NC}"
echo -e "${GREEN}           完成               ${NC}"
echo -e "${CYAN}=================================${NC}" 
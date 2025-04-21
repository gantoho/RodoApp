# Rodo - 美观的待办事项管理

Rodo 是一个使用 Rust 和 egui 编写的待办事项管理应用程序，具有美观的界面和简单的操作。

## 功能特点

- 待办事项的创建、编辑和删除
- 按优先级排序和过滤任务
- 支持任务标签管理
- 主题切换（明亮、暗黑、夕阳、海洋、森林）
- 自动保存功能
- 支持中文显示

## 开发指南

### 环境要求

- Rust 工具链（推荐使用 rustup 安装）
- Cargo 包管理器

### 构建与运行

您可以使用以下方法构建和运行项目：

#### 使用 Cargo 命令

```bash
# 清理构建
cargo clean

# 调试模式构建
cargo build

# 运行开发版本
cargo run

# 构建发布版本
cargo build --release

# 运行发布版本
cargo run --release
```

#### 使用便捷脚本

我们提供了两种脚本以简化开发流程：

1. **Windows 批处理脚本 (rebuild.cmd)**:
   
   ```bash
   # 标准构建和运行
   rebuild.cmd
   
   # 清理后构建和运行
   rebuild.cmd -c
   
   # 发布模式构建和运行
   rebuild.cmd -r
   
   # 清理后以发布模式构建和运行
   rebuild.cmd -c -r
   
   # 显示帮助信息
   rebuild.cmd -h
   ```

2. **PowerShell 脚本 (run.ps1)**:
   
   ```powershell
   # 标准构建和运行
   ./run.ps1
   
   # 清理后构建和运行
   ./run.ps1 -Clean
   
   # 发布模式构建和运行
   ./run.ps1 -Release
   
   # 清理后以发布模式构建和运行
   ./run.ps1 -Clean -Release
   
   # 显示帮助信息
   ./run.ps1 -Help
   ```

## 打包说明

我们提供了对Windows和Android平台的打包支持。

### Windows打包

Windows平台打包使用`cargo-bundle`进行，生成可安装的应用程序包。

```powershell
# 使用PowerShell脚本打包
./scripts/windows_build.ps1

# 清理后打包
./scripts/windows_build.ps1 -Clean

# 显示帮助信息
./scripts/windows_build.ps1 -Help
```

打包完成后，可以在以下位置找到构建结果：
- Windows安装包: `target/release/bundle/windows/`
- ZIP分发包: `target/Rodo-[版本号]-windows.zip`

### Android打包

Android平台打包使用`cargo-apk`进行，生成APK文件。需要预先安装Android NDK并设置环境变量。

```bash
# 在Linux/macOS上运行打包脚本
./scripts/android_build.sh

# 清理后打包
./scripts/android_build.sh -c

# 显示帮助信息
./scripts/android_build.sh -h
```

打包完成后，可以在以下位置找到构建结果：
- Android APK: `target/Rodo-[版本号]-android.apk`

**注意事项:**
- Android打包需要设置`ANDROID_NDK_HOME`环境变量
- 初次打包可能需要下载额外的依赖项，请确保网络连接正常
- 应用需要Android 6.0 (API 23)或更高版本

## 字体说明

应用默认使用 Noto Sans SC Regular 字体显示中文。您需要将字体文件放入正确的位置：

1. 下载 NotoSansSC-Regular.otf 字体文件（可从 Google Noto Fonts 项目获取）
2. 将字体文件放入 `assets/fonts/` 目录中
3. 文件路径应为 `assets/fonts/NotoSansSC-Regular.otf`

如果字体文件不存在，程序将尝试使用系统字体。

### 使用字体下载脚本

我们提供了便捷的脚本来下载所需的字体文件：

**Windows (PowerShell)**:
```powershell
.\scripts\download_fonts.ps1
```

**Linux/macOS**:
```bash
./scripts/download_fonts.sh
```

这些脚本会自动下载并放置字体文件到正确位置。

## 项目结构

- **src/**: 源代码目录
  - **main.rs**: 程序入口点
  - **lib.rs**: 库入口点，提供跨平台支持
  - **app.rs**: 应用程序状态和实现
  - **ui.rs**: 用户界面实现
  - **todo.rs**: 待办事项数据结构和功能
  - **theme.rs**: 应用主题定义
  - **ui_components.rs**: UI 组件（简化实现）
  - **ui_todo_edit.rs**: 待办事项编辑组件（简化实现）
  - **ui_settings.rs**: 设置界面组件（简化实现）
- **assets/**: 资源文件（如字体和图标）
- **scripts/**: 辅助脚本

## 许可证

此项目遵循 MIT 许可证。 
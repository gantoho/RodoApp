use std::env;
use std::path::Path;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=assets/");
    println!("cargo:rerun-if-changed=build.rs");
    
    // 获取crate类型，判断是否为可执行文件
    let crate_type = env::var("CARGO_CRATE_TYPE").unwrap_or_default();
    
    // 在Windows上设置GUI子系统，避免显示控制台窗口
    // 但只适用于可执行文件，不适用于库文件
    #[cfg(target_os = "windows")]
    if crate_type == "bin" {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    }
    
    // 确保target/release/assets目录存在
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();
    
    // 在开发模式下，不需要复制资源文件
    if profile != "release" {
        return;
    }
    
    // 确保输出目录存在
    let assets_output_dir = target_dir.join("assets");
    
    if !assets_output_dir.exists() {
        fs::create_dir_all(&assets_output_dir).expect("无法创建资源目录");
    }
    
    // 获取assets目录
    let assets_dir = Path::new("assets");
    if assets_dir.exists() && assets_dir.is_dir() {
        copy_dir_recursive(assets_dir, &assets_output_dir).expect("无法复制资源文件");
    } else {
        println!("找不到assets目录，跳过资源复制");
    }
    
    // 在Windows上添加图标资源
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icons/icon.ico")
            .set_language(0x0804) // 简体中文
            .set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="asInvoker" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
<compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
        <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
        <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}" />
        <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}" />
        <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}" />
        <supportedOS Id="{e2011457-1546-43c5-a5fe-008deee3d3f0}" />
    </application>
</compatibility>
<application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
        <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
        <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
</application>
</assembly>
            "#);
        
        if let Err(e) = res.compile() {
            eprintln!("无法编译Windows资源文件: {}", e);
        }
    }
    
    println!("资源文件复制完成");
}

// 递归复制目录
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
} 
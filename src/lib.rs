mod app;
mod theme;
mod todo;
mod ui;

use app::RodoApp;
use eframe::egui;

#[cfg(target_os = "android")]
mod android {
    extern crate ndk_glue;

    #[no_mangle]
    #[cfg(target_os = "android")]
    fn android_main(app: android_activity::AndroidApp) {
        use super::*;
        android_activity::init(app);
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Info)
                .with_tag("rodo"),
        );

        let options = eframe::NativeOptions {
            resizable: false,
            renderer: eframe::Renderer::Wgpu,
            hardware_acceleration: eframe::HardwareAcceleration::Preferred,
            multisampling: 4,
            vsync: true,
            ..Default::default()
        };

        eframe::run_native(
            "Rodo - 待办事项管理",
            options,
            Box::new(|cc| {
                // 设置中文字体
                setup_custom_fonts(&cc.egui_ctx);
                
                Box::new(RodoApp::new(cc))
            }),
        ).unwrap();
    }
}

/// 启动应用程序
pub fn run_app() -> Result<(), eframe::Error> {
    // 设置硬件加速选项
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 700.0)),
        min_window_size: Some(egui::vec2(600.0, 400.0)),
        renderer: eframe::Renderer::Wgpu,  // 显式指定使用wgpu
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        multisampling: 4,  // 启用抗锯齿
        vsync: true,  // 启用垂直同步
        ..Default::default()
    };
    
    // 显示启动信息
    println!("正在启动Rodo，使用wgpu渲染后端...");
    
    eframe::run_native(
        "Rodo - 美观的待办事项管理",
        options,
        Box::new(|cc| {
            // 设置中文字体
            setup_custom_fonts(&cc.egui_ctx);
            
            Box::new(RodoApp::new(cc))
        }),
    )
}

/// 设置支持中文字体
pub fn setup_custom_fonts(ctx: &egui::Context) {
    // 所有平台统一使用同一个字体文件
    let mut fonts = egui::FontDefinitions::default();
    
    // 使用指定的字体文件
    let font_name = "Noto Sans SC";
    
    // 尝试多个可能的字体文件路径
    let font_paths = [
        "assets/fonts/NotoSansSC-Regular.otf",
        "../assets/fonts/NotoSansSC-Regular.otf",
        "rodo/assets/fonts/NotoSansSC-Regular.otf",
        "./assets/fonts/NotoSansSC-Regular.otf",
    ];
    
    // 尝试加载字体文件
    let mut font_loaded = false;
    
    for font_path in font_paths.iter() {
        match std::fs::read(font_path) {
            Ok(font_data) => {
                // 加载字体成功
                fonts.font_data.insert(
                    font_name.to_string(),
                    egui::FontData::from_owned(font_data),
                );
                
                // 在各种文本样式中使用这种字体
                for (_text_style, font_list) in fonts.families.iter_mut() {
                    font_list.insert(0, font_name.to_string());
                }
                
                println!("已加载中文字体: {} (从 {})", font_name, font_path);
                font_loaded = true;
                break;
            },
            Err(_) => continue,
        }
    }
    
    // 如果所有路径都失败，则回退到系统字体
    if !font_loaded {
        eprintln!("无法加载字体文件，尝试使用系统字体");
        
        // 尝试使用系统字体
        #[cfg(target_os = "windows")]
        {
            setup_fonts_windows(ctx);
            return;
        }
        
        #[cfg(target_os = "android")]
        {
            setup_fonts_android(ctx);
            return;
        }
    }
    
    // 应用字体配置
    ctx.set_fonts(fonts);
}

/// 设置Windows字体
#[cfg(target_os = "windows")]
fn setup_fonts_windows(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 字体列表：按优先级排列的系统中文字体
    let font_choices = [
        "Microsoft YaHei UI", 
        "Microsoft YaHei",
        "SimHei",
        "SimSun",
        "NSimSun",
        "DengXian",
        "FangSong",
    ];
    
    // 尝试添加字体
    for font_name in &font_choices {
        if let Some(font_data) = load_system_font(font_name) {
            fonts.font_data.insert(
                font_name.to_string(),
                egui::FontData::from_owned(font_data),
            );
            
            // 在各种文本样式中使用这种字体
            for (_text_style, font_list) in fonts.families.iter_mut() {
                font_list.push(font_name.to_string());
            }
            
            // 找到一个可用字体后就停止
            break;
        }
    }
    
    ctx.set_fonts(fonts);
}

/// 设置Android字体
#[cfg(target_os = "android")]
fn setup_fonts_android(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Android 内置中文字体
    let font_choices = [
        "/system/fonts/NotoSansCJK-Regular.ttc",
        "/system/fonts/DroidSansFallback.ttf",
    ];
    
    // 尝试添加字体
    for font_path in &font_choices {
        if let Ok(font_data) = std::fs::read(font_path) {
            let font_name = format!("Android-{}", font_path.split('/').last().unwrap_or("Font"));
            
            fonts.font_data.insert(
                font_name.clone(),
                egui::FontData::from_owned(font_data),
            );
            
            // 在各种文本样式中使用这种字体
            for (_text_style, font_list) in fonts.families.iter_mut() {
                font_list.push(font_name.clone());
            }
            
            // 找到一个可用字体后就停止
            break;
        }
    }
    
    ctx.set_fonts(fonts);
}

/// 设置默认字体
#[cfg(not(any(target_os = "windows", target_os = "android")))]
fn setup_fonts_default(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 默认使用系统内置的无衬线字体
    let font_name = "Default Sans";
    fonts.font_data.insert(
        font_name.to_string(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansSC-Regular.otf")),
    );
    
    // 在各种文本样式中使用这种字体
    for (_text_style, font_list) in fonts.families.iter_mut() {
        font_list.push(font_name.to_string());
    }
    
    ctx.set_fonts(fonts);
}

/// 加载Windows系统字体
#[cfg(target_os = "windows")]
fn load_system_font(font_name: &str) -> Option<Vec<u8>> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use winapi::um::winuser::{GetDC, ReleaseDC};
    
    unsafe {
        let hdc = GetDC(null_mut());
        if hdc.is_null() {
            return None;
        }
        
        // 转换字符串为宽字符
        let wide_name: Vec<u16> = OsStr::new(font_name)
            .encode_wide()
            .chain(once(0))
            .collect();
            
        // 获取字体数据
        use winapi::um::wingdi::{EnumFontFamiliesExW, LOGFONTW, DEFAULT_CHARSET};
        
        struct FontContext {
            found: bool,
            data: Vec<u8>,
        }
        
        let mut font_ctx = FontContext {
            found: false,
            data: Vec::new(),
        };
        
        // 定义字体枚举回调函数
        extern "system" fn enum_font_proc(
            lpelfe: *const winapi::um::wingdi::LOGFONTW,
            _: *const winapi::um::wingdi::TEXTMETRICW,
            _: winapi::shared::minwindef::DWORD,
            lparam: winapi::shared::minwindef::LPARAM,
        ) -> i32 {
            unsafe {
                let ctx = &mut *(lparam as *mut FontContext);
                
                // 尝试获取字体数据
                let font_data = get_font_data(lpelfe);
                if !font_data.is_empty() {
                    ctx.data = font_data;
                    ctx.found = true;
                    return 0; // 停止枚举
                }
            }
            1 // 继续枚举
        }
        
        // 设置LOGFONT结构体
        let mut logfont: LOGFONTW = std::mem::zeroed();
        logfont.lfCharSet = DEFAULT_CHARSET as u8;
        
        // 将字体名填入结构体
        let max_copy = std::cmp::min(wide_name.len(), logfont.lfFaceName.len() - 1);
        logfont.lfFaceName[..max_copy].copy_from_slice(&wide_name[..max_copy]);
        
        // 枚举字体
        EnumFontFamiliesExW(
            hdc,
            &mut logfont,
            Some(enum_font_proc),
            &mut font_ctx as *mut _ as winapi::shared::minwindef::LPARAM,
            0,
        );
        
        ReleaseDC(null_mut(), hdc);
        
        if font_ctx.found {
            Some(font_ctx.data)
        } else {
            None
        }
    }
}

/// 获取字体数据
#[cfg(target_os = "windows")]
unsafe fn get_font_data(logfont: *const winapi::um::wingdi::LOGFONTW) -> Vec<u8> {
    use winapi::um::wingdi::{
        CreateFontIndirectW, DeleteObject, SelectObject, 
        GetFontData
    };
    use winapi::um::winuser::{GetDC, ReleaseDC};
    
    let hdc = GetDC(std::ptr::null_mut());
    if hdc.is_null() {
        return Vec::new();
    }
    
    // 使用LOGFONT创建实际字体
    let hfont = CreateFontIndirectW(logfont);
    if hfont.is_null() {
        ReleaseDC(std::ptr::null_mut(), hdc);
        return Vec::new();
    }
    
    // 选择字体到DC
    let old_font = SelectObject(hdc, hfont as _);
    
    // 获取字体数据大小
    let size = GetFontData(hdc, 0, 0, std::ptr::null_mut(), 0);
    let mut result = Vec::new();
    
    if size != 0xFFFFFFFF && size > 0 {
        // 分配内存并获取字体数据
        result.resize(size as usize, 0);
        let bytes_read = GetFontData(hdc, 0, 0, result.as_mut_ptr() as _, size);
        
        if bytes_read == 0xFFFFFFFF || bytes_read == 0 {
            result.clear();
        }
    }
    
    // 清理资源
    SelectObject(hdc, old_font);
    DeleteObject(hfont as _);
    ReleaseDC(std::ptr::null_mut(), hdc);
    
    result
} 
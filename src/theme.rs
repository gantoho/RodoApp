use egui::{Color32, Rounding, Stroke, Vec2, style::Margin, Visuals};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用主题类型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ThemeType {
    Light,
    Dark,
    Sunset,
    Ocean,
    Forest,
    Custom,
}

impl ThemeType {
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            ThemeType::Light => "明亮",
            ThemeType::Dark => "暗黑",
            ThemeType::Sunset => "夕阳",
            ThemeType::Ocean => "海洋",
            ThemeType::Forest => "森林",
            ThemeType::Custom => "自定义",
        }
    }

    #[allow(dead_code)]
    pub fn all() -> Vec<ThemeType> {
        vec![
            ThemeType::Light,
            ThemeType::Dark,
            ThemeType::Sunset,
            ThemeType::Ocean,
            ThemeType::Forest,
        ]
    }
}

/// 应用主题
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub theme_type: ThemeType,
    pub background: Color32,
    pub card_background: Color32,
    pub accent: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub selection: Color32,
}

impl Theme {
    /// 创建亮色主题
    #[allow(dead_code)]
    pub fn light() -> Self {
        Self {
            theme_type: ThemeType::Light,
            background: Color32::from_rgb(245, 245, 250),
            card_background: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(66, 133, 244),
            text: Color32::from_rgb(33, 33, 33),
            text_secondary: Color32::from_rgb(102, 102, 102),
            success: Color32::from_rgb(76, 175, 80),
            warning: Color32::from_rgb(255, 152, 0),
            error: Color32::from_rgb(244, 67, 54),
            selection: Color32::from_rgb(232, 240, 254),
        }
    }

    /// 创建暗色主题
    #[allow(dead_code)]
    pub fn dark() -> Self {
        Self {
            theme_type: ThemeType::Dark,
            background: Color32::from_rgb(30, 30, 35),
            card_background: Color32::from_rgb(45, 45, 50),
            accent: Color32::from_rgb(130, 170, 255),
            text: Color32::from_rgb(230, 230, 230),
            text_secondary: Color32::from_rgb(180, 180, 180),
            success: Color32::from_rgb(105, 220, 120),
            warning: Color32::from_rgb(255, 190, 90),
            error: Color32::from_rgb(255, 100, 100),
            selection: Color32::from_rgb(55, 70, 100),
        }
    }

    /// 创建夕阳主题
    #[allow(dead_code)]
    pub fn sunset() -> Self {
        Self {
            theme_type: ThemeType::Sunset,
            background: Color32::from_rgb(35, 25, 40),
            card_background: Color32::from_rgb(50, 35, 55),
            accent: Color32::from_rgb(255, 140, 120),
            text: Color32::from_rgb(240, 230, 230),
            text_secondary: Color32::from_rgb(200, 180, 190),
            success: Color32::from_rgb(140, 220, 170),
            warning: Color32::from_rgb(255, 190, 130),
            error: Color32::from_rgb(255, 120, 120),
            selection: Color32::from_rgb(80, 60, 90),
        }
    }

    /// 创建海洋主题
    #[allow(dead_code)]
    pub fn ocean() -> Self {
        Self {
            theme_type: ThemeType::Ocean,
            background: Color32::from_rgb(20, 40, 60),
            card_background: Color32::from_rgb(35, 55, 75),
            accent: Color32::from_rgb(100, 210, 255),
            text: Color32::from_rgb(230, 240, 250),
            text_secondary: Color32::from_rgb(170, 190, 210),
            success: Color32::from_rgb(100, 220, 200),
            warning: Color32::from_rgb(255, 190, 140),
            error: Color32::from_rgb(255, 130, 150),
            selection: Color32::from_rgb(50, 80, 110),
        }
    }

    /// 创建森林主题
    #[allow(dead_code)]
    pub fn forest() -> Self {
        Self {
            theme_type: ThemeType::Forest,
            background: Color32::from_rgb(30, 45, 35),
            card_background: Color32::from_rgb(45, 60, 50),
            accent: Color32::from_rgb(120, 200, 130),
            text: Color32::from_rgb(230, 240, 230),
            text_secondary: Color32::from_rgb(180, 200, 180),
            success: Color32::from_rgb(140, 230, 150),
            warning: Color32::from_rgb(230, 200, 110),
            error: Color32::from_rgb(230, 120, 110),
            selection: Color32::from_rgb(60, 85, 65),
        }
    }

    /// 基于主题类型获取对应主题
    #[allow(dead_code)]
    pub fn from_type(theme_type: ThemeType) -> Self {
        match theme_type {
            ThemeType::Light => Self::light(),
            ThemeType::Dark => Self::dark(),
            ThemeType::Sunset => Self::sunset(),
            ThemeType::Ocean => Self::ocean(),
            ThemeType::Forest => Self::forest(),
            ThemeType::Custom => Self::dark(), // 自定义模式默认使用暗黑主题作为基础
        }
    }
    
    /// 应用主题到egui上下文
    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // 设置基本视觉效果
        let mut visuals = if self.is_dark() {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        
        // 自定义主要颜色
        visuals.window_fill = self.card_background;
        visuals.panel_fill = self.background;
        visuals.faint_bg_color = self.background;
        visuals.extreme_bg_color = self.background;
        
        // 文本颜色无法直接设置，使用新版本的API
        
        // 选中与互动颜色
        visuals.selection.bg_fill = self.selection;
        visuals.selection.stroke = Stroke::new(1.0, self.accent);
        
        // 活跃状态
        visuals.widgets.active.bg_fill = self.card_background;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.text);
        
        // 非活跃状态
        visuals.widgets.inactive.bg_fill = self.background;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.text_secondary.linear_multiply(0.5));
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        
        // 鼠标停留状态
        visuals.widgets.hovered.bg_fill = self.background.linear_multiply(1.1);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text);
        
        // 圆角
        let rounding = Rounding::same(6.0);
        visuals.widgets.noninteractive.rounding = rounding;
        visuals.widgets.inactive.rounding = rounding;
        visuals.widgets.hovered.rounding = rounding;
        visuals.widgets.active.rounding = rounding;
        visuals.window_rounding = rounding;
        
        // 设置样式和间距
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.window_margin = Margin::same(16.0);
        style.spacing.button_padding = Vec2::new(8.0, 4.0);
        
        // 应用更改
        style.visuals = visuals;
        ctx.set_style(style);
    }
    
    /// 判断是否为暗色主题
    pub fn is_dark(&self) -> bool {
        match self.theme_type {
            ThemeType::Light => false,
            _ => true,
        }
    }
    
    /// 保存主题设置到文件
    pub fn save(&self) -> Result<(), String> {
        let path = Theme::get_theme_file_path()?;
        let serialized = serde_json::to_string(self).map_err(|e| format!("序列化主题失败: {}", e))?;
        std::fs::write(path, serialized).map_err(|e| format!("写入主题文件失败: {}", e))?;
        Ok(())
    }
    
    /// 从文件加载主题设置
    pub fn load() -> Self {
        match Theme::try_load() {
            Ok(theme) => theme,
            Err(_) => Self::default(), // 如果加载失败，使用默认主题
        }
    }
    
    /// 尝试从文件加载主题设置
    fn try_load() -> Result<Self, String> {
        let path = Theme::get_theme_file_path()?;
        if !path.exists() {
            return Err("主题文件不存在".to_string());
        }
        
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("读取主题文件失败: {}", e))?;
            
        serde_json::from_str(&data)
            .map_err(|e| format!("解析主题JSON失败: {}", e))
    }
    
    /// 获取主题文件路径
    fn get_theme_file_path() -> Result<PathBuf, String> {
        let app_dirs = match directories::ProjectDirs::from("com", "rodo", "rodo") {
            Some(dirs) => dirs,
            None => return Err("无法获取应用数据目录".to_string()),
        };
        
        let data_dir = app_dirs.data_dir();
        std::fs::create_dir_all(data_dir).map_err(|e| format!("无法创建数据目录: {}", e))?;
        
        Ok(data_dir.join("theme.json"))
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
} 
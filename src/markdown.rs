use egui::{
    Color32, FontId, RichText, TextFormat as EguiTextFormat,
    Stroke, Ui, ScrollArea, Label, FontFamily, CursorIcon
};
use pulldown_cmark::{Parser, Event, Tag, HeadingLevel, CodeBlockKind};
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightLines;
use std::fs;
use std::path::Path;
use open;

/// 加载Markdown文件
pub fn load_markdown_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("无法读取Markdown文件: {}", e))
}

/// 获取目录中的所有Markdown文件
pub fn get_markdown_files(dir_path: &Path) -> Result<Vec<String>, String> {
    if !dir_path.is_dir() {
        return Err(format!("指定的路径不是目录: {}", dir_path.display()));
    }
    
    let mut markdown_files = Vec::new();
    
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    // 检查是否是文件且扩展名为.md或.markdown
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if ext_str == "md" || ext_str == "markdown" {
                                // 添加相对路径（相对于目录）
                                if let Some(file_name) = path.file_name() {
                                    markdown_files.push(file_name.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
            
            // 对文件名进行排序
            markdown_files.sort();
            
            Ok(markdown_files)
        },
        Err(err) => Err(format!("无法读取目录: {}", err))
    }
}

/// 获取目录中的所有子目录
pub fn get_subdirectories(dir_path: &Path) -> Result<Vec<String>, String> {
    if !dir_path.is_dir() {
        return Err(format!("指定的路径不是目录: {}", dir_path.display()));
    }
    
    let mut subdirs = Vec::new();
    
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    // 检查是否是目录
                    if path.is_dir() {
                        if let Some(dir_name) = path.file_name() {
                            subdirs.push(dir_name.to_string_lossy().to_string());
                        }
                    }
                }
            }
            
            // 对目录名进行排序
            subdirs.sort();
            
            Ok(subdirs)
        },
        Err(err) => Err(format!("无法读取目录: {}", err))
    }
}

/// 渲染Markdown内容
pub fn render_markdown(ui: &mut Ui, content: &str, is_dark: bool) {
    // 创建解析器
    let parser = Parser::new(content);
    
    // 初始化语法高亮
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let theme = if is_dark {
        &theme_set.themes["base16-ocean.dark"]
    } else {
        &theme_set.themes["base16-eighties.light"] 
    };
    
    // 显示内容
    let mut current_code_block = String::new();
    let mut code_language = String::new();
    let mut in_code_block = false;
    
    // 当前文本缓冲和格式
    let mut current_text = String::new();
    let mut current_format = TextFormat::Normal;
    
    // 设置颜色
    let normal_color = get_text_color(is_dark);
    let code_bg_color = get_code_background(is_dark);
    let link_color = if ui.ctx().style().visuals.dark_mode {
        Color32::from_rgb(100, 149, 237) // 淡蓝色在深色主题
    } else {
        Color32::from_rgb(0, 0, 238) // 标准蓝色在浅色主题
    };
    
    // 辅助函数：刷新当前文本
    let mut flush_text = |ui: &mut Ui, text: &mut String, format: &TextFormat| {
        if !text.is_empty() {
            match format {
                TextFormat::Normal => {
                    ui.label(text.clone());
                },
                TextFormat::Heading(level) => {
                    // 这里我们根据级别设置不同大小的标题
                    let mut font_size = match level {
                        1 => 28.0,
                        2 => 24.0,
                        3 => 20.0,
                        4 => 18.0,
                        5 => 16.0,
                        _ => 14.0,
                    };
                    
                    let color = heading_style_to_color(*level, ui.visuals().dark_mode);
                    
                    ui.add(Label::new(
                        RichText::new(text.clone())
                            .size(font_size)
                            .color(color)
                            .strong()
                    ));
                },
                TextFormat::Strong => {
                    ui.add(Label::new(
                        RichText::new(text.clone()).strong()
                    ));
                },
                TextFormat::Emphasis => {
                    ui.add(Label::new(
                        RichText::new(text.clone()).italics()
                    ));
                },
                TextFormat::Code => {
                    let background_color = get_code_background(ui.visuals().dark_mode);
                    let text_color = get_text_color(ui.visuals().dark_mode);
                    
                    ui.add(Label::new(
                        RichText::new(text.clone())
                            .family(FontFamily::Monospace)
                            .background_color(background_color)
                            .color(text_color)
                    ));
                },
                TextFormat::Link(url) => {
                    let link_color = get_link_color(ui.visuals().dark_mode);
                    
                    let response = ui.add(Label::new(
                        RichText::new(text.clone())
                            .color(link_color)
                            .underline()
                    ));
                    
                    if response.clicked() {
                        if let Err(e) = open::that(url) {
                            eprintln!("Failed to open URL: {}", e);
                        }
                    }
                    
                    // 使鼠标悬停时显示为手型指针
                    response.on_hover_cursor(CursorIcon::PointingHand);
                }
            }
            text.clear();
        }
    };
    
    // 处理事件流
    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, _, _)) => {
                // 刷新之前的文本
                flush_text(ui, &mut current_text, &current_format);
                
                current_format = TextFormat::Heading(match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                });
                
                ui.add_space(10.0);
            },
            Event::End(Tag::Heading(_, _, _)) => {
                // 渲染标题文本
                flush_text(ui, &mut current_text, &current_format);
                current_format = TextFormat::Normal;
                ui.add_space(8.0);
            },
            Event::Start(Tag::Paragraph) => {
                flush_text(ui, &mut current_text, &current_format);
                ui.add_space(4.0);
            },
            Event::End(Tag::Paragraph) => {
                flush_text(ui, &mut current_text, &current_format);
                ui.add_space(4.0);
            },
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_text(ui, &mut current_text, &current_format);
                in_code_block = true;
                current_code_block.clear();
                
                if let CodeBlockKind::Fenced(lang) = kind {
                    code_language = lang.to_string();
                }
            },
            Event::End(Tag::CodeBlock(_)) => {
                // 渲染代码块
                if !current_code_block.is_empty() {
                    let mut highlighter = match syntax_set.find_syntax_by_extension(&code_language) {
                        Some(syntax) => HighlightLines::new(syntax, theme),
                        None => HighlightLines::new(syntax_set.find_syntax_plain_text(), theme),
                    };
                    
                    // 添加代码块背景
                    let frame = egui::Frame::none()
                        .fill(code_bg_color)
                        .inner_margin(egui::Margin::same(8.0))
                        .rounding(egui::Rounding::same(4.0));
                    
                    frame.show(ui, |ui| {
                        // 分行处理代码高亮
                        for line in current_code_block.lines() {
                            if let Ok(ranges) = highlighter.highlight_line(line, &syntax_set) {
                                let mut line_text = String::new();
                                let mut fragments = Vec::new();
                                
                                for (style, text) in ranges {
                                    let color = style_to_color(style);
                                    fragments.push((text, color));
                                }
                                
                                ui.horizontal(|ui| {
                                    for (text, color) in fragments {
                                        ui.label(RichText::new(text).monospace().color(color));
                                    }
                                });
                            } else {
                                // 如果高亮失败，直接显示原始文本
                                ui.label(RichText::new(line).monospace().color(normal_color));
                            }
                        }
                    });
                }
                in_code_block = false;
            },
            Event::Start(Tag::List(_)) => {
                flush_text(ui, &mut current_text, &current_format);
                ui.add_space(4.0);
            },
            Event::End(Tag::List(_)) => {
                flush_text(ui, &mut current_text, &current_format);
                ui.add_space(4.0);
            },
            Event::Start(Tag::Item) => {
                flush_text(ui, &mut current_text, &current_format);
                // 添加列表项前缀
                current_text.push_str("• ");
            },
            Event::End(Tag::Item) => {
                flush_text(ui, &mut current_text, &current_format);
                ui.end_row();  // 确保每个列表项都在新行
            },
            Event::Code(text) => {
                flush_text(ui, &mut current_text, &current_format);
                
                // 内联代码采用单独的背景和前景色
                ui.label(
                    RichText::new(text.as_ref())
                        .monospace()
                        .color(normal_color)
                        .background_color(code_bg_color)
                );
            },
            Event::Text(text) => {
                if in_code_block {
                    current_code_block.push_str(&text);
                } else {
                    current_text.push_str(&text);
                }
            },
            Event::Start(Tag::Emphasis) => {
                flush_text(ui, &mut current_text, &current_format);
                current_format = TextFormat::Emphasis;
            },
            Event::End(Tag::Emphasis) => {
                flush_text(ui, &mut current_text, &current_format);
                current_format = TextFormat::Normal;
            },
            Event::Start(Tag::Strong) => {
                flush_text(ui, &mut current_text, &current_format);
                current_format = TextFormat::Strong;
            },
            Event::End(Tag::Strong) => {
                flush_text(ui, &mut current_text, &current_format);
                current_format = TextFormat::Normal;
            },
            Event::Start(Tag::BlockQuote) => {
                flush_text(ui, &mut current_text, &current_format);
                
                let quote_color = get_blockquote_color(is_dark);
                
                // 设置引用块颜色和样式
                ui.push_id("blockquote", |ui| {
                    ui.horizontal(|ui| {
                        // 添加左侧竖线
                        let stroke = Stroke::new(3.0, quote_color);
                        ui.add_space(2.0);
                        ui.painter().vline(
                            ui.min_rect().left() + 3.0, 
                            ui.min_rect().y_range(), 
                            stroke
                        );
                        ui.add_space(8.0);
                        
                        // 引用内容区域
                        ui.vertical(|ui| {
                            // 引用块内容将在其他事件中处理
                            ui.add_space(2.0);
                        });
                    });
                });
            },
            Event::End(Tag::BlockQuote) => {
                flush_text(ui, &mut current_text, &current_format);
                ui.end_row();
            },
            Event::Start(Tag::Link(_, url, _)) => {
                flush_text(ui, &mut current_text, &current_format);
                current_format = TextFormat::Link(url.to_string());
            },
            Event::End(Tag::Link(_, _, _)) => {
                // 对于链接，我们使用 flush_text 而不是添加新的标签
                flush_text(ui, &mut current_text, &current_format);
                current_format = TextFormat::Normal;
            },
            Event::SoftBreak => {
                current_text.push(' ');
            },
            Event::HardBreak => {
                flush_text(ui, &mut current_text, &current_format);
                ui.add_space(8.0);
            },
            Event::Rule => {
                flush_text(ui, &mut current_text, &current_format);
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
            },
            _ => {}
        }
    }
    
    // 确保最后的文本被刷新
    flush_text(ui, &mut current_text, &current_format);
}

/// 语法高亮样式转换为egui颜色
fn style_to_color(style: Style) -> Color32 {
    let r = style.foreground.r;
    let g = style.foreground.g;
    let b = style.foreground.b;
    Color32::from_rgb(r, g, b)
}

/// 根据标题级别和主题获取合适的颜色
fn heading_style_to_color(level: usize, is_dark: bool) -> egui::Color32 {
    match level {
        1 => if is_dark { egui::Color32::from_rgb(255, 175, 135) } 
             else { egui::Color32::from_rgb(180, 85, 20) },
        2 => if is_dark { egui::Color32::from_rgb(200, 175, 255) } 
             else { egui::Color32::from_rgb(100, 80, 175) },
        3 => if is_dark { egui::Color32::from_rgb(135, 215, 255) } 
             else { egui::Color32::from_rgb(35, 120, 175) },
        4 => if is_dark { egui::Color32::from_rgb(175, 255, 200) } 
             else { egui::Color32::from_rgb(50, 140, 90) },
        5 => if is_dark { egui::Color32::from_rgb(255, 200, 175) } 
             else { egui::Color32::from_rgb(175, 80, 50) },
        _ => if is_dark { egui::Color32::from_rgb(220, 220, 220) } 
             else { egui::Color32::from_rgb(60, 60, 60) },
    }
}

/// 获取文本颜色
fn get_text_color(is_dark: bool) -> Color32 {
    if is_dark {
        Color32::from_rgb(220, 220, 220)
    } else {
        Color32::from_rgb(32, 32, 32)
    }
}

/// 获取代码块背景颜色
fn get_code_background(is_dark: bool) -> Color32 {
    if is_dark {
        Color32::from_rgb(45, 45, 45)
    } else {
        Color32::from_rgb(245, 245, 245)
    }
}

/// 获取引用块颜色
fn get_blockquote_color(is_dark: bool) -> Color32 {
    if is_dark {
        Color32::from_rgb(100, 160, 200)
    } else {
        Color32::from_rgb(70, 130, 180)
    }
}

/// 获取链接颜色
fn get_link_color(is_dark: bool) -> Color32 {
    if is_dark {
        Color32::from_rgb(100, 149, 237) // 淡蓝色在深色主题
    } else {
        Color32::from_rgb(0, 0, 238) // 标准蓝色在浅色主题
    }
}

// 文本格式枚举
enum TextFormat {
    Normal,
    Heading(usize),
    Strong,
    Emphasis,
    Code,
    Link(String),
} 
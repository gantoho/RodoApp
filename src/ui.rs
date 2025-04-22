use crate::app::{ConfirmationAction, RodoApp, View};
use crate::theme::Theme;
use crate::todo::{Emoji, Priority, SubTask, Todo};
use crate::markdown;
use crate::globals::WINDOW_VISIBLE;
use egui::{Button, Color32, Layout, RichText, ScrollArea, Ui, Vec2};
use chrono::{DateTime, Local};
use uuid::Uuid;
use rfd::FileDialog;
use std::sync::atomic::Ordering;

/// 安全地截取字符串，避免在UTF-8字符边界处截断
fn truncate_string(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i >= max_chars {
            break;
        }
        result.push(c);
    }
    result.push_str("...");
    result
}

impl eframe::App for RodoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 响应窗口可见性变化
        if WINDOW_VISIBLE.load(Ordering::SeqCst) {
            // 确保窗口可见并聚焦
            ctx.request_repaint();
        }
        
        // 应用主题
        self.theme.apply_to_ctx(ctx);
        
        // 顶部面板
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                
                // 标志和标题
                ui.heading(RichText::new("Rodo").color(self.theme.accent));
                
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    // 设置按钮
                    if ui.button("⚙️").clicked() {
                        self.view = View::Settings;
                    }
                    
                    // Markdown按钮 - 在设置按钮前面
                    if ui.button("📄").clicked() {
                        self.view = View::MarkdownViewer;
                        
                        // 如果已经有保存的目录但文件列表为空，尝试重新加载目录中的文件
                        if let Some(dir_path) = &self.current_markdown_directory {
                            if self.markdown_files.is_empty() {
                                if let Ok(files) = markdown::get_markdown_files(std::path::Path::new(dir_path)) {
                                    self.markdown_files = files;
                                }
                            }
                            
                            // 如果有保存的文件路径但内容为空，尝试加载文件内容
                            if let Some(file_path) = &self.current_markdown_path {
                                if self.markdown_content.is_empty() {
                                    let path = std::path::Path::new(file_path);
                                    if path.exists() {
                                        if let Ok(content) = markdown::load_markdown_file(path) {
                                            self.markdown_content = content;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // 任务列表按钮
                    if ui.button("📝").clicked() {
                        self.view = View::List;
                    }
                    
                    ui.add_space(16.0);
                });
            });
            
            ui.add_space(8.0);
        });
        
        // 主要内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.view {
                View::List => self.render_todo_list(ui),
                View::AddTodo => self.render_add_todo(ui),
                View::EditTodo => self.render_edit_todo(ui),
                View::Settings => self.render_settings(ui),
                View::Stats => self.render_stats(ui),
                View::Tags => self.render_tags(ui),
                View::About => self.render_about(ui),
                View::MarkdownViewer => self.render_markdown_viewer(ui),
            }
        });
        
        // 显示确认对话框
        if self.show_confirmation {
            self.render_confirmation_dialog(ctx);
        }
        
        // 每帧自动保存（如果有修改）
        self.save();
    }
}

impl RodoApp {
    /// 渲染待办事项列表
    fn render_todo_list(&mut self, ui: &mut Ui) {
        // 标题和操作按钮
        ui.horizontal(|ui| {
            ui.heading("待办事项");
            
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                // 标签管理按钮
                if ui.button("🏷️ 标签").clicked() {
                    self.view = View::Tags;
                }
                
                // 删除Markdown预览器按钮
                // if ui.button("📄 Markdown").clicked() {
                //     self.view = View::MarkdownViewer;
                // }
                
                // 优先级排序按钮
                {
                    let sort_text = match self.todo_list.priority_sort {
                        Some(true) => "🔽 优先级高",   // 从高到低
                        Some(false) => "🔼 优先级低",  // 从低到高
                        None => "⏺️ 时间排序",       // 默认按时间排序
                    };
                    
                    // 创建一个特殊风格的按钮
                    let mut button = egui::Button::new(RichText::new(sort_text).strong());
                    
                    // 根据排序状态设置按钮样式
                    if self.todo_list.priority_sort.is_some() {
                        // 激活状态下使用填充色
                        button = button.fill(ui.visuals().selection.bg_fill)
                                      .stroke(egui::Stroke::new(2.0, ui.visuals().selection.stroke.color))
                                      .rounding(egui::Rounding::same(8.0));
                    } else {
                        // 未激活状态下使用特殊的边框和轻微填充
                        let accent_color = self.theme.accent;
                        button = button.fill(Color32::from_rgba_premultiplied(
                                    accent_color.r(), accent_color.g(), accent_color.b(), 20))
                                 .stroke(egui::Stroke::new(2.0, accent_color))
                                 .rounding(egui::Rounding::same(8.0));
                    }
                    
                    // 添加额外的内边距使按钮更大
                    if ui.add_sized(Vec2::new(130.0, 32.0), button).clicked() {
                        // 切换排序状态：时间排序 -> 优先级高 -> 优先级低 -> 时间排序
                        self.todo_list.priority_sort = match self.todo_list.priority_sort {
                            None => Some(true),        // 时间排序 -> 优先级高
                            Some(true) => Some(false), // 优先级高 -> 优先级低
                            Some(false) => None,       // 优先级低 -> 时间排序
                        };
                        self.modified = true;
                    }
                }
                
                ui.add_space(8.0);
                
                // 筛选选项 - 使用按钮替代复选框，以便更加醒目
                {
                    let filter_text = if self.todo_list.filter_completed {
                        "🔍 显示所有"
                    } else {
                        "🔍 隐藏已完成"
                    };
                    
                    // 创建一个特殊风格的按钮，使用更具有辨识度的样式
                    let mut button = egui::Button::new(RichText::new(filter_text).strong());
                    
                    // 当过滤器激活时使用不同的样式
                    if self.todo_list.filter_completed {
                        // 显示所有 - 使用蓝色调
                        let color = self.theme.accent.linear_multiply(1.2); // 使用主题的强调色，但稍微亮一点
                        button = button.fill(color)
                                      .stroke(egui::Stroke::new(2.0, self.theme.accent))
                                      .rounding(egui::Rounding::same(12.0));
                    } else {
                        // 隐藏已完成 - 使用绿色调
                        let color = self.theme.success.linear_multiply(0.8); // 使用主题的成功色，但稍微暗一点
                        button = button.fill(color)
                                 .stroke(egui::Stroke::new(2.0, self.theme.success))
                                 .rounding(egui::Rounding::same(12.0));
                    }
                    
                    // 使用特殊尺寸和样式，添加阴影效果使按钮看起来像是浮起来的
                    let response = ui.add_sized(Vec2::new(150.0, 36.0), button);
                    
                    // 绘制微弱的阴影效果
                    let rect = response.rect;
                    let shadow_offset = 3.0;
                    let shadow_rect = egui::Rect::from_min_max(
                        rect.min + Vec2::new(shadow_offset, shadow_offset),
                        rect.max + Vec2::new(shadow_offset, shadow_offset),
                    );
                    
                    // 在按钮后面绘制阴影
                    ui.painter().rect_filled(
                        shadow_rect,
                        egui::Rounding::same(12.0),
                        Color32::from_rgba_premultiplied(0, 0, 0, 30), // 半透明黑色阴影
                    );
                    
                    // 处理点击事件
                    if response.clicked() {
                        self.todo_list.filter_completed = !self.todo_list.filter_completed;
                        self.modified = true;
                    }
                }
                
                ui.add_space(16.0);
            });
        });
        
        ui.separator();
        
        // 显示活跃标签过滤器（如果有）
        if !self.todo_list.active_tags.is_empty() {
            ui.horizontal(|ui| {
                ui.label("筛选标签:");
                // 先收集需要移除的索引
                let mut indices_to_remove = Vec::new();
                
                for (idx, tag) in self.todo_list.active_tags.iter().enumerate() {
                    if ui.button(format!("🏷️ {}", tag)).clicked() {
                        // 记录要移除的标签索引，而不是直接修改
                        indices_to_remove.push(idx);
                    }
                }
                
                // 在循环外移除标签
                if !indices_to_remove.is_empty() {
                    // 从后往前移除，以避免索引失效
                    for idx in indices_to_remove.iter().rev() {
                        self.todo_list.active_tags.remove(*idx);
                        self.modified = true;
                    }
                }
            });
            ui.add_space(8.0);
        }
        
        // 渲染任务列表
        let todos = self.todo_list.filtered_todos();
        
        if todos.is_empty() {
            // 显示空状态
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("没有待办事项");
                ui.add_space(8.0);
                if ui.button("添加任务").clicked() {
                    self.view = View::AddTodo;
                    self.new_todo = Todo::new(String::new());
                }
                ui.add_space(50.0);
            });
        } else {
            // 预先收集所有任务所需的信息
            let todo_infos: Vec<(String, String, bool, Priority, String, Vec<String>, usize, usize, DateTime<Local>, Option<DateTime<Local>>)> = todos
                .iter()
                .map(|todo| {
                    // 计算子任务完成数量
                    let completed_subtasks = todo.subtasks.iter().filter(|st| st.completed).count();
                    let total_subtasks = todo.subtasks.len();
                    
                    // 提取表情符号
                    let emoji = match &todo.emoji {
                        Emoji::None => String::new(),
                        Emoji::CheckMark => "✅".to_string(),
                        Emoji::Star => "⭐".to_string(),
                        Emoji::Heart => "❤️".to_string(),
                        Emoji::Fire => "🔥".to_string(),
                        Emoji::Book => "📚".to_string(),
                        Emoji::Music => "🎵".to_string(),
                        Emoji::Sport => "🏃".to_string(),
                        Emoji::Shopping => "🛒".to_string(),
                        Emoji::Work => "💼".to_string(),
                        Emoji::Family => "👪".to_string(),
                        Emoji::Health => "🏥".to_string(),
                        Emoji::Travel => "✈️".to_string(),
                        Emoji::Custom(ref s) => s.clone(),
                    };
                    
                    // 返回元组(id, title, completed, priority, emoji, tags, completed_subtasks, total_subtasks, created_at, completed_at)
                    (
                        todo.id.clone(),
                        todo.title.clone(),
                        todo.completed,
                        todo.priority.clone(),
                        emoji,
                        todo.tags.clone(),
                        completed_subtasks,
                        total_subtasks,
                        todo.created_at,
                        todo.completed_at.clone()
                    )
                })
                .collect();
            
            // 显示任务列表
            ScrollArea::vertical().show(ui, |ui| {
                for (id, title, completed, priority, emoji, tags, completed_subtasks, total_subtasks, created_at, completed_at) in todo_infos {
                    ui.add_space(4.0);
                    
                    // 任务卡片背景
                    let card_bg = if completed {
                        ui.visuals().faint_bg_color
                    } else {
                        ui.visuals().panel_fill
                    };
                    
                    // 任务卡片边框颜色（基于优先级）
                    let priority_color = match priority {
                        Priority::Low => egui::Color32::from_rgb(76, 175, 80),      // 绿色
                        Priority::Medium => egui::Color32::from_rgb(255, 193, 7),    // 黄色
                        Priority::High => egui::Color32::from_rgb(255, 87, 34),      // 橙色
                        Priority::Critical => egui::Color32::from_rgb(244, 67, 54),  // 红色
                    };
                    
                    // 绘制任务卡片
                    egui::Frame::none()
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0, priority_color))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // 完成状态勾选框
                                let mut is_completed = completed;
                                if ui.checkbox(&mut is_completed, "").clicked() {
                                    if let Some(t) = self.todo_list.todos.get_mut(&id) {
                                        // 使用新的set_completed方法
                                        t.set_completed(is_completed);
                                        self.modified = true;
                                    }
                                }
                                
                                // 任务内容区域
                                ui.vertical(|ui| {
                                    // 标题行（包含表情符号和标题）
                                    ui.horizontal(|ui| {
                                        // 表情符号
                                        if !emoji.is_empty() {
                                            ui.label(&emoji);
                                        }
                                        
                                        // 标题，点击可编辑
                                        let title_text = if completed {
                                            RichText::new(&title).strikethrough()
                                        } else {
                                            RichText::new(&title)
                                        };
                                        
                                        if ui.add(egui::Label::new(title_text).sense(egui::Sense::click())).clicked() {
                                            self.editing_todo_id = Some(id.clone());
                                            self.view = View::EditTodo;
                                        }
                                    });
                                    
                                    // 如果有描述，尝试获取
                                    if let Some(todo) = self.todo_list.todos.get(&id) {
                                        if !todo.description.is_empty() {
                                            let desc = if todo.description.chars().count() > 50 {
                                                truncate_string(&todo.description, 47)
                                            } else {
                                                todo.description.clone()
                                            };
                                            ui.label(RichText::new(desc).italics().small());
                                        }
                                    }
                                    
                                    // 显示创建时间和完成时间
                                    ui.horizontal(|ui| {
                                        let date_color = self.theme.text_secondary;
                                        
                                        // 创建时间
                                        let created_text = format!("创建: {}", Todo::format_date_time(&created_at));
                                        ui.label(RichText::new(created_text).color(date_color).small());
                                        
                                        ui.add_space(8.0);
                                        
                                        // 完成时间（如果已完成）
                                        if let Some(completed_time) = completed_at {
                                            let completed_text = format!("完成: {}", Todo::format_date_time(&completed_time));
                                            ui.label(RichText::new(completed_text).color(date_color).small());
                                        }
                                    });
                                    
                                    // 显示标签（如果有）
                                    if !tags.is_empty() {
                                        ui.horizontal(|ui| {
                                            for tag in &tags {
                                                let tag_button = ui.button(format!("🏷️ {}", tag));
                                                if tag_button.clicked() {
                                                    // 检查标签是否已经在活跃标签中
                                                    let is_active = self.todo_list.active_tags.contains(tag);
                                                    if !is_active {
                                                        self.todo_list.active_tags.push(tag.clone());
                                                        self.modified = true;
                                                    }
                                                }
                                            }
                                        });
                                    }
                                    
                                    // 显示子任务进度（如果有子任务）
                                    if total_subtasks > 0 {
                                        ui.label(format!("子任务: {}/{}", completed_subtasks, total_subtasks));
                                    }
                                });
                                
                                // 右边显示优先级标签
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    let priority_text = match priority {
                                        Priority::Low => "低",
                                        Priority::Medium => "中",
                                        Priority::High => "高",
                                        Priority::Critical => "紧急",
                                    };
                                    
                                    ui.label(RichText::new(priority_text).color(priority_color).small());
                                });
                            });
                        });
                    
                    ui.add_space(4.0);
                }
            });
        }
        
        // 添加浮动的添加任务按钮
        ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(
                egui::pos2(ui.available_rect_before_wrap().right() - 150.0, ui.available_rect_before_wrap().bottom() - 60.0),
                egui::vec2(130.0, 50.0)
            ),
            |ui| {
                // 创建一个特殊风格的"添加任务"按钮
                let mut add_button = Button::new(
                    RichText::new("➕ 添加任务")
                        .strong()
                        .size(18.0)
                );
                
                // 使用主题的强调色作为背景，使按钮更加醒目
                let accent_color = self.theme.accent;
                add_button = add_button
                    .fill(accent_color)
                    .stroke(egui::Stroke::new(1.5, Color32::WHITE))
                    .rounding(egui::Rounding::same(8.0));
                
                // 添加额外的内边距和阴影效果
                if ui.add_sized(Vec2::new(130.0, 46.0), add_button).clicked() {
                    self.view = View::AddTodo;
                    self.new_todo = Todo::new(String::new());
                }
            }
        );
    }
    
    /// 渲染添加新待办事项页面
    fn render_add_todo(&mut self, ui: &mut Ui) {
        // 添加滚动区域，确保所有编辑字段都可见
        ScrollArea::vertical().show(ui, |ui| {
            ui.heading(if self.editing_todo_id.is_some() { "编辑任务" } else { "添加任务" });
            ui.separator();
            
            ui.add_space(16.0);
            
            // emoji选择器
            ui.horizontal(|ui| {
                ui.label("选择表情:");
                ui.add_space(10.0);
                
                let emojis = vec!["📝", "📌", "🔍", "📅", "📚", "💼", "🎯", "🏆", "⚙️", "🔧", "💡", "🎨", "🔔"];
                
                for emoji in emojis {
                    let is_selected = match &self.new_todo.emoji {
                        Emoji::Custom(s) if s == emoji => true,
                        _ => false
                    };
                    if ui.selectable_label(is_selected, emoji).clicked() {
                        self.new_todo.emoji = Emoji::Custom(emoji.to_string());
                        self.modified = true;
                    }
                }
            });
            
            ui.add_space(8.0);
            
            // 优先级选择器
            ui.horizontal(|ui| {
                ui.label("优先级:");
                ui.add_space(10.0);
                
                let priorities = [
                    (Priority::Low, "低", Color32::from_rgb(100, 180, 100)),
                    (Priority::Medium, "中", Color32::from_rgb(180, 180, 100)),
                    (Priority::High, "高", Color32::from_rgb(180, 100, 100)),
                    (Priority::Critical, "紧急", Color32::from_rgb(180, 50, 50)),
                ];
                
                for (priority, label, color) in &priorities {
                    let is_selected = self.new_todo.priority == *priority;
                    let mut button = Button::new(*label);
                    
                    if is_selected {
                        button = button.fill(*color);
                    } else {
                        button = button.stroke((1.0, *color));
                    }
                    
                    if ui.add(button).clicked() {
                        self.new_todo.priority = priority.clone();
                        self.modified = true;
                    }
                }
            });
            
            ui.add_space(16.0);
            
            // 任务标题
            ui.horizontal(|ui| {
                ui.label("标题:");
                ui.add(egui::TextEdit::singleline(&mut self.new_todo.title).hint_text("任务标题"));
            });
            
            ui.add_space(8.0);
            
            // 完成状态
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.new_todo.completed, "标记为完成");
                
                // 如果是编辑现有任务，显示创建/完成时间
                if self.editing_todo_id.is_some() {
                    ui.add_space(16.0);
                    let date_color = self.theme.text_secondary;
                    
                    // 创建时间
                    ui.label(RichText::new(format!("创建于: {}", Todo::format_date_time(&self.new_todo.created_at))).color(date_color).small());
                    
                    // 如果已完成，显示完成时间
                    if self.new_todo.completed {
                        if let Some(completed_time) = &self.new_todo.completed_at {
                            ui.add_space(8.0);
                            ui.label(RichText::new(format!("完成于: {}", Todo::format_date_time(completed_time))).color(date_color).small());
                        }
                    }
                }
            });
            
            ui.add_space(8.0);
            
            // 任务描述
            ui.horizontal(|ui| {
                ui.label("描述:");
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.new_todo.description)
                            .hint_text("任务描述（可选）")
                            .desired_width(f32::INFINITY)
                            .desired_rows(3)
                    );
                });
            });
            
            ui.add_space(8.0);
            
            // 标签编辑
            ui.label("标签:");
            
            // 显示已选标签
            ui.horizontal(|ui| {
                let tags = self.new_todo.tags.clone();
                for (i, tag) in tags.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("🏷️ {}", tag));
                        if ui.small_button("❌").clicked() {
                            self.new_todo.tags.remove(i);
                            self.modified = true;
                        }
                    });
                }
            });
            
            // 添加新标签输入框 - 放在单独的一行
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("添加:");
                ui.add(egui::TextEdit::singleline(&mut self.temp_tag_input).hint_text("输入标签名称"));
                
                let can_add = !self.temp_tag_input.trim().is_empty() && 
                    !self.new_todo.tags.contains(&self.temp_tag_input.trim().to_string());
                
                if ui.add_enabled(can_add, egui::Button::new("添加标签")).clicked() {
                    self.new_todo.tags.push(self.temp_tag_input.trim().to_string());
                    self.temp_tag_input.clear();
                    self.modified = true;
                }
            });
            
            
            // 显示已有标签供选择
            ui.add_space(4.0);
            ui.label("选择已有标签:");
            
            // 收集所有已存在的标签（不在当前任务中的）
            let mut all_tags = std::collections::HashSet::new();
            for todo in self.todo_list.todos.values() {
                for tag in &todo.tags {
                    all_tags.insert(tag.clone());
                }
            }
            
            // 移除当前任务已有的标签
            for tag in &self.new_todo.tags {
                all_tags.remove(tag);
            }
            
            // 如果有可选的已存在标签，显示它们
            if !all_tags.is_empty() {
                // 将HashSet转换为Vec以便排序
                let mut available_tags: Vec<String> = all_tags.into_iter().collect();
                available_tags.sort(); // 字母顺序排序
                
                ui.horizontal_wrapped(|ui| {
                    for tag in available_tags {
                        if ui.button(format!("🏷️ {}", tag)).clicked() {
                            if !self.new_todo.tags.contains(&tag) {
                                self.new_todo.tags.push(tag);
                                self.modified = true;
                            }
                        }
                    }
                });
            } else {
                ui.label(RichText::new("(暂无其他标签)").italics().small());
            }
            
            ui.add_space(16.0);
            
            // 子任务编辑
            ui.collapsing("子任务", |ui| {
                // 使用状态变量避免借用冲突
                let mut subtask_index_to_remove = None;
                let mut subtask_index_to_toggle = None;
                
                // 显示现有子任务
                for (i, subtask) in self.new_todo.subtasks.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let mut completed = subtask.completed;
                        if ui.checkbox(&mut completed, "").clicked() {
                            subtask_index_to_toggle = Some(i);
                        }
                        
                        ui.label(&subtask.title);
                        
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("删除").clicked() {
                                subtask_index_to_remove = Some(i);
                            }
                        });
                    });
                }
                
                // 处理子任务状态变更
                if let Some(index) = subtask_index_to_toggle {
                    let mut subtask = self.new_todo.subtasks[index].clone();
                    subtask.completed = !subtask.completed;
                    self.new_todo.subtasks[index] = subtask;
                    self.modified = true;
                }
                
                // 处理子任务删除
                if let Some(index) = subtask_index_to_remove {
                    self.new_todo.subtasks.remove(index);
                    self.modified = true;
                }
                
                // 添加新子任务
                ui.horizontal(|ui| {
                    ui.label("新子任务:");
                    ui.add(egui::TextEdit::singleline(&mut self.temp_input).hint_text("子任务标题"));
                    
                    let can_add = !self.temp_input.trim().is_empty();
                    
                    if ui.add_enabled(can_add, egui::Button::new("添加")).clicked() {
                        self.new_todo.subtasks.push(SubTask::new(self.temp_input.trim().to_string()));
                        self.temp_input.clear();
                        self.modified = true;
                    }
                });
            });
            
            ui.add_space(16.0);
            
            // 按钮区域
            ui.horizontal(|ui| {
                // 取消按钮
                if ui.button("取消").clicked() {
                    self.view = View::List;
                    self.new_todo = Todo::new("".to_string());
                    self.temp_tag_input.clear();
                    self.temp_input.clear();
                    self.editing_todo_id = None;
                }
                
                // 只有当任务标题不为空时才启用保存按钮
                let can_save = !self.new_todo.title.trim().is_empty();
                
                if ui.add_enabled(can_save, egui::Button::new("保存")).clicked() {
                    let mut todo = self.new_todo.clone();
                    
                    if self.editing_todo_id.is_none() {
                        // 创建新任务
                        todo.id = format!("todo-{}", Uuid::new_v4());
                        todo.created_at = chrono::Local::now();
                        // 设置完成状态，这会自动处理完成时间
                        if todo.completed {
                            todo.set_completed(true);
                        }
                        self.todo_list.todos.insert(todo.id.clone(), todo);
                    } else if let Some(todo_id) = &self.editing_todo_id {
                        // 更新现有任务
                        if let Some(existing_todo) = self.todo_list.todos.get_mut(todo_id) {
                            // 保留创建时间
                            todo.created_at = existing_todo.created_at.clone();
                            // 保留完成时间，只有状态改变时才更新
                            if todo.completed != existing_todo.completed {
                                todo.set_completed(todo.completed);
                            } else {
                                todo.completed_at = existing_todo.completed_at.clone();
                            }
                            *existing_todo = todo;
                        }
                    }
                    
                    self.view = View::List;
                    self.new_todo = Todo::new("".to_string());
                    self.temp_tag_input.clear();
                    self.temp_input.clear();
                    self.editing_todo_id = None;
                    self.modified = true;
                }
            });
        });
    }
    
    /// 渲染编辑待办事项页面
    fn render_edit_todo(&mut self, ui: &mut Ui) {
        // 获取正在编辑的任务
        let editing_id = match &self.editing_todo_id {
            Some(id) => id.clone(),
            None => {
                self.view = View::List;
                return;
            }
        };
        
        // 获取待编辑的任务
        let todo = match self.todo_list.todos.get(&editing_id) {
            Some(todo) => todo.clone(),
            None => {
                self.view = View::List;
                return;
            }
        };
        
        ui.heading("编辑任务");
        ui.separator();
        
        ui.add_space(16.0);
        
        // 创建滚动区域以容纳所有编辑字段
        ScrollArea::vertical().show(ui, |ui| {
            // 表情符号选择
            ui.horizontal(|ui| {
                ui.label("表情符号:");
                ui.horizontal_wrapped(|ui| {
                    let all_emojis = [
                        (Emoji::None, "无"),
                        (Emoji::CheckMark, "✅"),
                        (Emoji::Star, "⭐"),
                        (Emoji::Heart, "❤️"),
                        (Emoji::Fire, "🔥"),
                        (Emoji::Book, "📚"),
                        (Emoji::Music, "🎵"),
                        (Emoji::Sport, "🏃"),
                        (Emoji::Shopping, "🛒"),
                        (Emoji::Work, "💼"),
                        (Emoji::Family, "👪"),
                        (Emoji::Health, "🏥"),
                        (Emoji::Travel, "✈️"),
                    ];
                    
                    for (emoji_type, emoji_char) in all_emojis.iter() {
                        if ui.selectable_label(matches!(&todo.emoji, e if std::mem::discriminant(e) == std::mem::discriminant(emoji_type)), *emoji_char).clicked() {
                            if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                                t.emoji = emoji_type.clone();
                                self.modified = true;
                            }
                        }
                    }
                });
            });
            
            ui.add_space(8.0);
            
            // 完成状态
            let mut completed = todo.completed;
            if ui.checkbox(&mut completed, "标记为完成").clicked() {
                if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                    t.set_completed(completed);
                    self.modified = true;
                }
            }
            
            ui.add_space(8.0);
            
            // 显示任务的创建和完成时间
            ui.horizontal(|ui| {
                let date_color = self.theme.text_secondary;
                
                // 创建时间
                ui.label(RichText::new(format!("创建于: {}", Todo::format_date_time(&todo.created_at))).color(date_color));
                
                ui.add_space(16.0);
                
                // 完成时间（如果已完成）
                if let Some(completed_time) = &todo.completed_at {
                    ui.label(RichText::new(format!("完成于: {}", Todo::format_date_time(completed_time))).color(date_color));
                }
            });
            
            ui.add_space(12.0);
            
            // 优先级选择
            ui.horizontal(|ui| {
                ui.label("优先级:");
                
                let priority_options = [
                    (Priority::Low, "低", egui::Color32::from_rgb(76, 175, 80)),
                    (Priority::Medium, "中", egui::Color32::from_rgb(255, 193, 7)),
                    (Priority::High, "高", egui::Color32::from_rgb(255, 87, 34)),
                    (Priority::Critical, "紧急", egui::Color32::from_rgb(244, 67, 54)),
                ];
                
                for (prio_type, prio_text, prio_color) in priority_options.iter() {
                    if ui.selectable_label(
                        matches!(&todo.priority, p if std::mem::discriminant(p) == std::mem::discriminant(prio_type)),
                        RichText::new(*prio_text).color(*prio_color)
                    ).clicked() {
                        if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                            t.priority = prio_type.clone();
                            self.modified = true;
                        }
                    }
                }
            });
            
            ui.add_space(12.0);
            
            // 任务标题
            ui.label("任务标题 *");
            let mut title = todo.title.clone();
            if ui.text_edit_singleline(&mut title).changed() {
                if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                    t.title = title;
                    self.modified = true;
                }
            }
            
            ui.add_space(12.0);
            
            // 任务描述
            ui.label("任务描述");
            let mut description = todo.description.clone();
            if ui.text_edit_multiline(&mut description).changed() {
                if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                    t.description = description;
                    self.modified = true;
                }
            }
            
            ui.add_space(12.0);
            
            // 标签管理
            ui.label("标签");
            
            // 显示现有标签
            ui.horizontal_wrapped(|ui| {
                let mut tags = todo.tags.clone();
                let mut tags_to_remove = Vec::new();
                
                for (i, tag) in tags.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("🏷️ {}", tag));
                        if ui.small_button("❌").clicked() {
                            tags_to_remove.push(i);
                            self.modified = true;
                        }
                    });
                }
                
                // 移除标记的标签
                for i in tags_to_remove.iter().rev() {
                    tags.remove(*i);
                }
                
                // 更新标签
                if tags != todo.tags {
                    if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                        t.tags = tags;
                    }
                }
            });
            
            // 添加新标签 - 放在单独的一行
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("新标签:");
                if ui.text_edit_singleline(&mut self.temp_tag_input).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !self.temp_tag_input.trim().is_empty() {
                        if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                            t.tags.push(self.temp_tag_input.trim().to_string());
                            self.modified = true;
                            self.temp_tag_input.clear();
                        }
                    }
                }
                
                if ui.button("添加").clicked() && !self.temp_tag_input.trim().is_empty() {
                    if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                        t.tags.push(self.temp_tag_input.trim().to_string());
                        self.modified = true;
                        self.temp_tag_input.clear();
                    }
                }
            });

            
            // 显示已有标签供选择
            ui.add_space(4.0);
            ui.label("选择已有标签:");
            
            // 收集所有已存在的标签（不在当前任务中的）
            let mut all_tags = std::collections::HashSet::new();
            for t in self.todo_list.todos.values() {
                for tag in &t.tags {
                    all_tags.insert(tag.clone());
                }
            }
            
            // 移除当前任务已有的标签
            for tag in &todo.tags {
                all_tags.remove(tag);
            }
            
            // 如果有可选的已存在标签，显示它们
            if !all_tags.is_empty() {
                // 将HashSet转换为Vec以便排序
                let mut available_tags: Vec<String> = all_tags.into_iter().collect();
                available_tags.sort(); // 字母顺序排序
                
                ui.horizontal_wrapped(|ui| {
                    for tag in available_tags {
                        if ui.button(format!("🏷️ {}", tag)).clicked() {
                            if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                                if !t.tags.contains(&tag) {
                                    t.tags.push(tag);
                                    self.modified = true;
                                }
                            }
                        }
                    }
                });
            } else {
                ui.label(RichText::new("(暂无其他标签)").italics().small());
            }
            
            ui.add_space(12.0);
            
            // 子任务管理
            ui.collapsing("子任务", |ui| {
                let mut subtasks = todo.subtasks.clone();
                let mut tasks_to_remove = Vec::new();
                
                // 显示现有子任务
                for (i, subtask) in subtasks.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut subtask.completed, "").clicked() {
                            self.modified = true;
                        }
                        
                        let mut title = subtask.title.clone();
                        if ui.text_edit_singleline(&mut title).changed() {
                            subtask.title = title;
                            self.modified = true;
                        }
                        
                        if ui.small_button("❌").clicked() {
                            tasks_to_remove.push(i);
                            self.modified = true;
                        }
                    });
                }
                
                // 移除标记的子任务
                for i in tasks_to_remove.iter().rev() {
                    subtasks.remove(*i);
                }
                
                // 更新子任务
                if subtasks != todo.subtasks {
                    if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                        t.subtasks = subtasks;
                    }
                }
                
                // 添加新子任务
                ui.horizontal(|ui| {
                    ui.label("新子任务:");
                    if ui.text_edit_singleline(&mut self.temp_input).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.temp_input.trim().is_empty() {
                            if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                                t.subtasks.push(SubTask::new(self.temp_input.trim().to_string()));
                                self.modified = true;
                                self.temp_input.clear();
                            }
                        }
                    }
                    
                    if ui.button("添加").clicked() && !self.temp_input.trim().is_empty() {
                        if let Some(t) = self.todo_list.todos.get_mut(&editing_id) {
                            t.subtasks.push(SubTask::new(self.temp_input.trim().to_string()));
                            self.modified = true;
                            self.temp_input.clear();
                        }
                    }
                });
                
                ui.add_space(4.0);
            });
            
            ui.add_space(16.0);
            
            // 按钮区域
            ui.horizontal(|ui| {
                if ui.button("返回").clicked() {
                    self.view = View::List;
                }
                
                if ui.button("删除").clicked() {
                    self.show_confirm(
                        "确定要删除此任务吗？",
                        ConfirmationAction::DeleteTodo(editing_id.clone()),
                    );
                }
                
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("保存").clicked() {
                        self.view = View::List;
                    }
                });
            });
        });
    }
    
    /// 渲染设置页面
    fn render_settings(&mut self, ui: &mut Ui) {
        ui.heading("设置");
        ui.separator();
        
        ui.add_space(16.0);
        
        // 主题设置区域
        ui.heading("主题设置");
        
        // 显示当前主题
        ui.label(format!("当前主题: {}", match self.theme.theme_type {
            crate::theme::ThemeType::Light => "明亮",
            crate::theme::ThemeType::Dark => "暗黑",
            crate::theme::ThemeType::Sunset => "日落",
            crate::theme::ThemeType::Ocean => "海洋",
            crate::theme::ThemeType::Forest => "森林",
            crate::theme::ThemeType::Custom => "自定义",
            crate::theme::ThemeType::Preset(ref name) => name,
        }));
        
        ui.add_space(8.0);
        
        // 主题选择器
        ui.horizontal_wrapped(|ui| {
            let theme_options = [
                (crate::theme::ThemeType::Light, "明亮", Color32::from_rgb(240, 240, 240)),
                (crate::theme::ThemeType::Dark, "暗黑", Color32::from_rgb(50, 50, 60)),
                (crate::theme::ThemeType::Sunset, "日落", Color32::from_rgb(255, 180, 120)),
                (crate::theme::ThemeType::Ocean, "海洋", Color32::from_rgb(100, 160, 200)),
                (crate::theme::ThemeType::Forest, "森林", Color32::from_rgb(120, 180, 120)),
            ];
            
            for (theme_type, name, color) in &theme_options {
                let is_selected = matches!(&self.theme.theme_type, t if std::mem::discriminant(t) == std::mem::discriminant(theme_type));
                
                let mut button = Button::new(*name);
                if is_selected {
                    button = button.fill(*color).stroke(egui::Stroke::new(2.0, self.theme.accent));
                } else {
                    button = button.fill(Color32::from_rgba_premultiplied(
                        color.r(), color.g(), color.b(), 40
                    )).stroke(egui::Stroke::new(1.0, *color));
                }
                
                // 添加圆角和内边距
                button = button.rounding(egui::Rounding::same(8.0));
                
                if ui.add_sized(Vec2::new(100.0, 40.0), button).clicked() && !is_selected {
                    // 改变主题并保存
                    let new_theme = match theme_type {
                        crate::theme::ThemeType::Light => crate::theme::Theme::light(),
                        crate::theme::ThemeType::Dark => crate::theme::Theme::dark(),
                        crate::theme::ThemeType::Sunset => crate::theme::Theme::sunset(),
                        crate::theme::ThemeType::Ocean => crate::theme::Theme::ocean(),
                        crate::theme::ThemeType::Forest => crate::theme::Theme::forest(),
                        _ => self.theme.clone(),
                    };
                    
                    // 调用app.rs中的set_theme方法
                    crate::app::RodoApp::set_theme(self, new_theme, ui.ctx());
                }
            }
            
            // 添加自定义主题按钮
            let mut custom_button = Button::new("自定义");
            let is_custom = matches!(self.theme.theme_type, crate::theme::ThemeType::Custom);
            
            if is_custom {
                custom_button = custom_button
                    .fill(self.theme.background)
                    .stroke(egui::Stroke::new(2.0, self.theme.accent));
            } else {
                custom_button = custom_button
                    .fill(Color32::from_rgba_premultiplied(180, 180, 180, 40))
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(150)));
            }
            
            custom_button = custom_button.rounding(egui::Rounding::same(8.0));
            
            if ui.add_sized(Vec2::new(100.0, 40.0), custom_button).clicked() {
                // 如果当前不是自定义主题，创建一个基于当前主题的自定义副本
                if !is_custom {
                    let mut custom_theme = self.theme.clone();
                    custom_theme.theme_type = crate::theme::ThemeType::Custom;
                    crate::app::RodoApp::set_theme(self, custom_theme, ui.ctx());
                }
            }
        });
        
        ui.add_space(16.0);
        
        // 自定义主题编辑器 - 只在自定义主题模式显示
        if matches!(self.theme.theme_type, crate::theme::ThemeType::Custom) {
            ui.collapsing("自定义主题编辑", |ui| {
                ui.add_space(8.0);
                
                // 创建一个临时主题以跟踪变化
                let mut theme = self.theme.clone();
                let mut theme_changed = false;
                
                // 背景颜色
                ui.horizontal(|ui| {
                    ui.label("背景颜色:");
                    let mut color = [
                        theme.background.r() as f32 / 255.0,
                        theme.background.g() as f32 / 255.0,
                        theme.background.b() as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        theme.background = Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        theme_changed = true;
                        // 立即应用变更
                        crate::app::RodoApp::set_theme(self, theme.clone(), ui.ctx());
                    }
                });
                
                // 卡片背景颜色
                ui.horizontal(|ui| {
                    ui.label("卡片背景:");
                    let mut color = [
                        theme.card_background.r() as f32 / 255.0,
                        theme.card_background.g() as f32 / 255.0,
                        theme.card_background.b() as f32 / 255.0,
                    ];
                    // 原本的方法，不使用自定义的扩展
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        theme.card_background = Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        theme_changed = true;
                        // 立即应用变更
                        crate::app::RodoApp::set_theme(self, theme.clone(), ui.ctx());
                    }
                });
                
                // 强调色
                ui.horizontal(|ui| {
                    ui.label("强调色:");
                    let mut color = [
                        theme.accent.r() as f32 / 255.0,
                        theme.accent.g() as f32 / 255.0,
                        theme.accent.b() as f32 / 255.0,
                    ];
                    // 简单的颜色编辑按钮
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        theme.accent = Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        theme_changed = true;
                        // 立即应用变更
                        crate::app::RodoApp::set_theme(self, theme.clone(), ui.ctx());
                    }
                });
                
                // 主文本颜色
                ui.horizontal(|ui| {
                    ui.label("主文本颜色:");
                    let mut color = [
                        theme.text.r() as f32 / 255.0,
                        theme.text.g() as f32 / 255.0,
                        theme.text.b() as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        theme.text = Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        theme_changed = true;
                        // 立即应用变更
                        crate::app::RodoApp::set_theme(self, theme.clone(), ui.ctx());
                    }
                });
                
                // 次要文本颜色
                ui.horizontal(|ui| {
                    ui.label("次要文本:");
                    let mut color = [
                        theme.text_secondary.r() as f32 / 255.0,
                        theme.text_secondary.g() as f32 / 255.0,
                        theme.text_secondary.b() as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        theme.text_secondary = Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        theme_changed = true;
                        // 立即应用变更
                        crate::app::RodoApp::set_theme(self, theme.clone(), ui.ctx());
                    }
                });
                
                // 成功颜色
                ui.horizontal(|ui| {
                    ui.label("成功颜色:");
                    let mut color = [
                        theme.success.r() as f32 / 255.0,
                        theme.success.g() as f32 / 255.0,
                        theme.success.b() as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        theme.success = Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        theme_changed = true;
                        // 立即应用变更
                        crate::app::RodoApp::set_theme(self, theme.clone(), ui.ctx());
                    }
                });
                
                // 警告颜色
                ui.horizontal(|ui| {
                    ui.label("警告颜色:");
                    let mut color = [
                        theme.warning.r() as f32 / 255.0,
                        theme.warning.g() as f32 / 255.0,
                        theme.warning.b() as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        theme.warning = Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        theme_changed = true;
                        // 立即应用变更
                        crate::app::RodoApp::set_theme(self, theme.clone(), ui.ctx());
                    }
                });
                
                // 错误颜色
                ui.horizontal(|ui| {
                    ui.label("错误颜色:");
                    let mut color = [
                        theme.error.r() as f32 / 255.0,
                        theme.error.g() as f32 / 255.0,
                        theme.error.b() as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        theme.error = Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        theme_changed = true;
                        // 立即应用变更
                        crate::app::RodoApp::set_theme(self, theme.clone(), ui.ctx());
                    }
                });
                
                // 选中颜色
                ui.horizontal(|ui| {
                    ui.label("选中颜色:");
                    let mut color = [
                        theme.selection.r() as f32 / 255.0,
                        theme.selection.g() as f32 / 255.0,
                        theme.selection.b() as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        theme.selection = Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        theme_changed = true;
                        // 立即应用变更
                        crate::app::RodoApp::set_theme(self, theme.clone(), ui.ctx());
                    }
                });
                
                ui.add_space(8.0);
                
                // 基于预设生成新的自定义主题
                ui.add_space(8.0);
                ui.label("从预设复制:");
                ui.horizontal(|ui| {
                    if ui.button("明亮").clicked() {
                        let mut new_theme = crate::theme::Theme::light();
                        new_theme.theme_type = crate::theme::ThemeType::Custom;
                        crate::app::RodoApp::set_theme(self, new_theme, ui.ctx());
                    }
                    if ui.button("暗黑").clicked() {
                        let mut new_theme = crate::theme::Theme::dark();
                        new_theme.theme_type = crate::theme::ThemeType::Custom;
                        crate::app::RodoApp::set_theme(self, new_theme, ui.ctx());
                    }
                    if ui.button("日落").clicked() {
                        let mut new_theme = crate::theme::Theme::sunset();
                        new_theme.theme_type = crate::theme::ThemeType::Custom;
                        crate::app::RodoApp::set_theme(self, new_theme, ui.ctx());
                    }
                    if ui.button("海洋").clicked() {
                        let mut new_theme = crate::theme::Theme::ocean();
                        new_theme.theme_type = crate::theme::ThemeType::Custom;
                        crate::app::RodoApp::set_theme(self, new_theme, ui.ctx());
                    }
                    if ui.button("森林").clicked() {
                        let mut new_theme = crate::theme::Theme::forest();
                        new_theme.theme_type = crate::theme::ThemeType::Custom;
                        crate::app::RodoApp::set_theme(self, new_theme, ui.ctx());
                    }
                });
                
                // 添加用户自定义的预设主题
                let preset_names = self.theme_presets.get_preset_names();
                if !preset_names.is_empty() {
                    ui.label("我的预设:");
                    ui.horizontal_wrapped(|ui| {
                        for name in preset_names {
                            let button = ui.button(&name);
                            if button.clicked() {
                                if let Err(err) = self.apply_theme_preset(&name, ui.ctx()) {
                                    eprintln!("应用主题预设失败: {}", err);
                                    // 显示错误消息
                                    self.show_confirm(
                                        &format!("应用主题预设失败: {}", err),
                                        crate::app::ConfirmationAction::ImportTodos, // 重用已有的确认动作类型
                                    );
                                }
                            }
                            
                            // 删除预设按钮
                            if button.secondary_clicked() || 
                               (button.clicked() && ui.input(|i| i.modifiers.shift)) {
                                self.show_confirm(
                                    &format!("确定要删除主题预设 \"{}\" 吗？", name),
                                    crate::app::ConfirmationAction::DeleteThemePreset(name.clone()),
                                );
                            }
                        }
                    });
                }
                
                // 保存当前自定义主题为预设
                ui.add_space(8.0);
                ui.label("保存为预设:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.temp_input)
                       .on_hover_text("输入预设名称");
                    
                    let can_save = !self.temp_input.trim().is_empty();
                    if ui.add_enabled(can_save, egui::Button::new("保存")).clicked() {
                        let name = self.temp_input.trim().to_string();
                        match self.save_theme_preset(name) {
                            Ok(_) => {
                                self.temp_input.clear();
                            },
                            Err(err) => {
                                eprintln!("保存主题预设失败: {}", err);
                                // 显示错误消息
                                self.show_confirm(
                                    &format!("保存主题预设失败: {}", err),
                                    crate::app::ConfirmationAction::ImportTodos, // 重用已有的确认动作类型
                                );
                            }
                        }
                    }
                });
            });
        }
        
        ui.add_space(16.0);
        
        // 数据导入导出区域
        ui.heading("数据管理");
        ui.add_space(8.0);
        
        ui.horizontal(|ui| {
            if ui.button("📤 导出任务").clicked() {
                self.export_todos_dialog();
            }
            
            if ui.button("📥 导入任务").clicked() {
                self.import_todos_dialog();
            }
            
            if ui.button("📥 合并导入").clicked() {
                self.merge_todos_dialog();
            }
        });
        
        ui.add_space(16.0);
        
        ui.heading("其他设置");
        
        // 添加关于按钮
        if ui.button("关于 Rodo").clicked() {
            self.view = View::About;
        }
        
        ui.add_space(16.0);
        
        ui.horizontal(|ui| {
            if ui.button("返回").clicked() {
                self.view = View::List;
            }
            
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                // 恢复初始状态按钮 - 使用警告色以表示危险操作
                let reset_button = Button::new(RichText::new("恢复初始状态").color(self.theme.error));
                if ui.add(reset_button).clicked() {
                    self.show_confirm(
                        "确定要恢复初始状态吗？这将清空所有数据并重置所有设置，此操作不可撤销！",
                        ConfirmationAction::ResetApp,
                    );
                }
                
                ui.add_space(8.0);
                
                if ui.button("重置默认主题").clicked() {
                    self.show_confirm(
                        "确定要重置为默认主题吗？",
                        ConfirmationAction::ResetSettings,
                    );
                }
            });
        });
    }
    
    /// 渲染统计页面
    fn render_stats(&mut self, ui: &mut Ui) {
        ui.heading("统计");
        ui.separator();
        
        ui.add_space(16.0);
        
        ui.label("统计功能尚未实现。");
        
        ui.add_space(16.0);
        
        if ui.button("返回").clicked() {
            self.view = View::List;
        }
    }
    
    /// 渲染标签管理页面
    fn render_tags(&mut self, ui: &mut Ui) {
        ui.heading("标签管理");
        ui.separator();
        
        ui.add_space(16.0);
        
        // 缓存活跃标签以避免UI闪烁
        let active_tags = self.todo_list.active_tags.clone();
        
        // 获取所有标签和使用次数
        let mut all_tags = std::collections::HashMap::new();
        for todo in self.todo_list.todos.values() {
            for tag in &todo.tags {
                *all_tags.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        // 标签排序：按使用次数降序
        let mut tag_counts: Vec<(String, usize)> = all_tags.into_iter().collect();
        tag_counts.sort_by(|a, b| {
            // 首先按使用次数降序排列
            let count_order = b.1.cmp(&a.1);
            // 当使用次数相同时，按标签名称字母顺序排序，保持稳定性
            if count_order == std::cmp::Ordering::Equal {
                a.0.cmp(&b.0)
            } else {
                count_order
            }
        });
        
        if tag_counts.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("暂无标签");
                ui.add_space(20.0);
            });
        } else {
            // 显示标签统计
            ui.label(format!("总共 {} 个标签", tag_counts.len()));
            ui.add_space(12.0);
            
            // 使用状态变量避免借用冲突
            let mut tag_to_delete = None;
            let mut tags_to_toggle = Vec::new();
            
            // 显示标签列表
            ScrollArea::vertical()
                .id_source("tags_scroll_area")  // 添加一个标识符确保稳定性
                .max_height(400.0)
                .show(ui, |ui| {
                    for (_i, (tag, count)) in tag_counts.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // 为每行标签创建一个唯一ID，使用标签内容而非索引
                            let tag_id = format!("tag_{}", tag);
                            
                            // 标签名称和使用次数
                            ui.label(format!("🏷️ {} ({})", tag, count));
                            
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                // 删除标签按钮，设置唯一ID
                                let delete_btn_id = ui.id().with(format!("{}_delete", tag_id));
                                if ui.push_id(delete_btn_id, |ui| {
                                    ui.button(egui::RichText::new("删除").text_style(egui::TextStyle::Body))
                                        .on_hover_text("删除此标签")
                                        .clicked()
                                }).inner {
                                    tag_to_delete = Some(tag.clone());
                                }
                                
                                // 标签筛选按钮 - 使用缓存的活跃标签列表
                                let is_active = active_tags.contains(tag);
                                let text = if is_active { "取消筛选" } else { "筛选" };
                                
                                let toggle_btn_id = ui.id().with(format!("{}_toggle", tag_id));
                                if ui.push_id(toggle_btn_id, |ui| {
                                    ui.selectable_label(is_active, text).clicked()
                                }).inner {
                                    tags_to_toggle.push(tag.clone());
                                }
                            });
                        });
                        
                        ui.separator();
                    }
                });
            
            // 在循环外处理标签操作，避免借用冲突
            if let Some(tag) = tag_to_delete {
                self.show_confirm(
                    &format!("确定要删除标签 \"{}\" 吗？这将从所有任务中移除该标签。", tag),
                    ConfirmationAction::DeleteTag(tag),
                );
            }
            
            // 批量处理标签切换，减少UI重绘
            if !tags_to_toggle.is_empty() {
                for tag in tags_to_toggle {
                    let is_active = active_tags.contains(&tag);
                    if is_active {
                        if let Some(pos) = self.todo_list.active_tags.iter().position(|t| t == &tag) {
                            self.todo_list.active_tags.remove(pos);
                        }
                    } else {
                        self.todo_list.active_tags.push(tag);
                    }
                }
                self.modified = true;
            }
            
            ui.add_space(8.0);
            
            // 新标签输入
            ui.horizontal(|ui| {
                ui.label("新标签:");
                let response = ui.add(egui::TextEdit::singleline(&mut self.temp_tag_input)
                    .hint_text("输入标签名称")
                    .id_source("new_tag_input"));  // 添加ID确保稳定性
                
                let can_add = !self.temp_tag_input.trim().is_empty() && 
                                !tag_counts.iter().any(|(t, _)| t == &self.temp_tag_input.trim());
                
                let add_clicked = ui.add_enabled(can_add, egui::Button::new("添加")).clicked();
                
                // 处理回车键或点击添加按钮
                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && can_add) || add_clicked {
                    // 创建新标签
                    if can_add {
                        // 实际上，标签只有在任务中使用才会存在，所以这里只是清空输入
                        self.temp_tag_input.clear();
                    }
                }
            });
        }
        
        ui.add_space(16.0);
        
        ui.horizontal(|ui| {
            if ui.button("返回").clicked() {
                self.view = View::List;
            }
            
            if !tag_counts.is_empty() {
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("清除所有筛选").clicked() {
                        self.todo_list.active_tags.clear();
                        self.modified = true;
                    }
                });
            }
        });
    }
    
    /// 渲染关于页面
    fn render_about(&mut self, ui: &mut Ui) {
        ui.heading("关于 Rodo");
        ui.separator();
        
        ui.add_space(16.0);
        
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            
            ui.heading("Rodo");
            ui.add_space(16.0);
            
            ui.label("待办事项管理工具");
            ui.add_space(8.0);
            
            // 从Cargo.toml获取的信息
            ui.label("版本: 0.0.1");
            ui.add_space(8.0);
            ui.label("开发者: github@xiuton@gantoho");
            ui.add_space(8.0);
            ui.label("描述: 一个由Rust，Egui构建的待办事项应用程序");
            ui.add_space(16.0);
            
            // 开源项目地址
            ui.horizontal(|ui| {
                ui.label("开源项目地址:");
                if ui.link("https://github.com/xiuton/RodoApp").clicked() {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Err(e) = Self::open_url("https://github.com/xiuton/RodoApp") {
                            eprintln!("无法打开URL: {}", e);
                        }
                    }
                }
            });
            
            ui.add_space(32.0);
        });
        
        if ui.button("返回").clicked() {
            self.view = View::List;
        }
    }
    
    /// 打开URL的辅助函数
    #[cfg(not(target_arch = "wasm32"))]
    fn open_url(url: &str) -> Result<(), std::io::Error> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("cmd")
                .args(["/c", "start", "", url])
                .spawn()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("open")
                .arg(url)
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("xdg-open")
                .arg(url)
                .spawn()?;
        }
        
        Ok(())
    }
    
    /// 显示导出任务对话框
    fn export_todos_dialog(&mut self) {
        // 使用rfd库打开文件保存对话框
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 默认文件名
            let default_filename = "todos_export.json";
            
            // 尝试打开文件保存对话框
            if let Some(path) = rfd::FileDialog::new()
                .set_title("选择导出文件保存位置")
                .set_file_name(default_filename)
                .add_filter("JSON文件", &["json"])
                .save_file()
            {
                match self.export_todos(&path) {
                    Ok(_) => {
                        // 显示成功消息
                        println!("成功导出任务到: {:?}", path);
                        // 创建一个确认对话框
                        self.show_confirm(
                            &format!("成功导出任务到: {}", path.display()),
                            ConfirmationAction::ImportTodos, // 使用已有的确认动作类型
                        );
                    },
                    Err(e) => {
                        // 显示错误消息
                        eprintln!("导出任务失败: {}", e);
                        // 创建一个错误对话框
                        self.show_confirm(
                            &format!("导出任务失败: {}", e),
                            ConfirmationAction::ImportTodos, // 使用已有的确认动作类型
                        );
                    }
                }
            }
        }
        
        // 如果无法打开文件选择对话框或在Web环境下，使用默认路径
        #[cfg(target_arch = "wasm32")]
        {
            // 创建一个固定的JSON文件保存路径
            let output_path = std::path::Path::new("todos_export.json");
            
            match self.export_todos(output_path) {
                Ok(_) => {
                    // 显示成功消息
                    println!("成功导出任务到: {:?}", output_path);
                    // 创建一个确认对话框
                    self.show_confirm(
                        &format!("成功导出任务到: {}", output_path.display()),
                        ConfirmationAction::ImportTodos, // 使用已有的确认动作类型
                    );
                },
                Err(e) => {
                    // 显示错误消息
                    eprintln!("导出任务失败: {}", e);
                    // 创建一个错误对话框
                    self.show_confirm(
                        &format!("导出任务失败: {}", e),
                        ConfirmationAction::ImportTodos, // 使用已有的确认动作类型
                    );
                }
            }
        }
    }
    
    /// 显示导入任务对话框
    fn import_todos_dialog(&mut self) {
        // 使用rfd库打开文件选择对话框
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 默认文件路径，如果不存在则使用当前目录
            let default_path = std::path::Path::new("todos_export.json");
            let default_dir = if default_path.exists() {
                default_path.parent().unwrap_or(std::path::Path::new("."))
            } else {
                std::path::Path::new(".")
            };
            
            // 尝试打开文件选择对话框
            if let Some(path) = rfd::FileDialog::new()
                .set_title("选择要导入的JSON文件")
                .set_directory(default_dir)
                .add_filter("JSON文件", &["json"])
                .pick_file()
            {
                // 提示确认，因为导入会覆盖现有任务
                self.show_confirm(
                    &format!("导入将从 {} 加载并覆盖当前所有任务，确定要继续吗？", path.display()),
                    ConfirmationAction::ImportTodos,
                );
                
                // 保存路径，等待确认后导入
                self.temp_input = path.to_string_lossy().to_string();
            }
        }
        
        // 如果无法打开文件选择对话框或在Web环境下，使用默认路径
        #[cfg(target_arch = "wasm32")]
        {
            let input_path = std::path::Path::new("todos_export.json");
            
            // 检查文件是否存在
            if input_path.exists() {
                // 提示确认，因为导入会覆盖现有任务
                self.show_confirm(
                    &format!("导入将从 {} 加载并覆盖当前所有任务，确定要继续吗？", input_path.display()),
                    ConfirmationAction::ImportTodos,
                );
                
                // 保存路径，等待确认后导入
                self.temp_input = input_path.to_string_lossy().to_string();
            } else {
                // 文件不存在，显示错误消息
                self.show_confirm(
                    &format!("找不到导入文件: {}，请先导出任务", input_path.display()),
                    ConfirmationAction::ImportTodos,
                );
            }
        }
    }
    
    /// 显示合并导入对话框
    fn merge_todos_dialog(&mut self) {
        // 使用rfd库打开文件选择对话框
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 默认文件路径，如果不存在则使用当前目录
            let default_path = std::path::Path::new("todos_export.json");
            let default_dir = if default_path.exists() {
                default_path.parent().unwrap_or(std::path::Path::new("."))
            } else {
                std::path::Path::new(".")
            };
            
            // 尝试打开文件选择对话框
            if let Some(path) = rfd::FileDialog::new()
                .set_title("选择要合并导入的JSON文件")
                .set_directory(default_dir)
                .add_filter("JSON文件", &["json"])
                .pick_file()
            {
                // 执行合并导入
                match self.merge_imported_todos(&path) {
                    Ok(count) => {
                        println!("成功导入 {} 个新任务", count);
                        // 创建一个确认对话框
                        self.show_confirm(
                            &format!("成功从 {} 导入 {} 个新任务", path.display(), count),
                            ConfirmationAction::ImportTodos, // 使用已有的确认动作类型
                        );
                    },
                    Err(e) => {
                        eprintln!("导入任务失败: {}", e);
                        // 创建一个错误对话框
                        self.show_confirm(
                            &format!("导入任务失败: {}", e),
                            ConfirmationAction::ImportTodos, // 使用已有的确认动作类型
                        );
                    }
                }
            }
        }
        
        // 如果无法打开文件选择对话框或在Web环境下，使用默认路径
        #[cfg(target_arch = "wasm32")]
        {
            let input_path = std::path::Path::new("todos_export.json");
            
            // 检查文件是否存在
            if input_path.exists() {
                match self.merge_imported_todos(input_path) {
                    Ok(count) => {
                        println!("成功导入 {} 个新任务", count);
                        // 创建一个确认对话框
                        self.show_confirm(
                            &format!("成功导入 {} 个新任务", count),
                            ConfirmationAction::ImportTodos, // 使用已有的确认动作类型
                        );
                    },
                    Err(e) => {
                        eprintln!("导入任务失败: {}", e);
                        // 创建一个错误对话框
                        self.show_confirm(
                            &format!("导入任务失败: {}", e),
                            ConfirmationAction::ImportTodos, // 使用已有的确认动作类型
                        );
                    }
                }
            } else {
                // 文件不存在，显示错误消息
                self.show_confirm(
                    &format!("找不到导入文件: {}，请先导出任务", input_path.display()),
                    ConfirmationAction::ImportTodos, // 使用已有的确认动作类型
                );
            }
        }
    }
    
    /// 渲染确认对话框
    fn render_confirmation_dialog(&mut self, ctx: &egui::Context) {
        // 保存导入路径，以避免借用冲突
        let import_path = if let Some(ConfirmationAction::ImportTodos) = &self.confirmation_action {
            self.temp_input.clone()
        } else {
            String::new()
        };
        
        // 保存标签名，以避免借用冲突
        let tag_to_delete = if let Some(ConfirmationAction::DeleteTag(_tag)) = &self.confirmation_action {
            _tag.clone()
        } else {
            String::new()
        };
        
        // 保存主题预设名，以避免借用冲突
        let preset_to_delete = if let Some(ConfirmationAction::DeleteThemePreset(_preset)) = &self.confirmation_action {
            _preset.clone()
        } else {
            String::new()
        };
        
        egui::Window::new("确认")
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(300.0, 150.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(&self.confirmation_message);
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            self.show_confirmation = false;
                            self.confirmation_action = None;
                        }
                        
                        if ui.button("确定").clicked() {
                            match self.confirmation_action.take() {
                                Some(ConfirmationAction::DeleteTodo(id)) => {
                                    self.delete_todo(&id);
                                },
                                Some(ConfirmationAction::DeleteAllCompleted) => {
                                    self.delete_all_completed();
                                },
                                Some(ConfirmationAction::ResetSettings) => {
                                    // 重置为默认主题并保存
                                    crate::app::RodoApp::set_theme(self, Theme::default(), ctx);
                                },
                                Some(ConfirmationAction::ResetApp) => {
                                    // 恢复应用程序到初始状态
                                    self.reset_app(ctx);
                                },
                                Some(ConfirmationAction::DeleteThemePreset(_preset)) => {
                                    // 使用事先保存的预设名，避免借用冲突
                                    if !preset_to_delete.is_empty() {
                                        if let Err(err) = self.delete_theme_preset(&preset_to_delete) {
                                            eprintln!("删除主题预设失败: {}", err);
                                            self.show_confirm(
                                                &format!("删除主题预设失败: {}", err),
                                                ConfirmationAction::ImportTodos, // 重用已有的确认动作类型
                                            );
                                        }
                                    }
                                },
                                Some(ConfirmationAction::ImportTodos) => {
                                    // 使用事先保存的路径，避免借用冲突
                                    if !import_path.is_empty() {
                                        let path = std::path::Path::new(&import_path);
                                        // 判断确认消息中是否包含"覆盖"，以区分常规导入和合并导入
                                        if self.confirmation_message.contains("覆盖") {
                                            // 常规导入（覆盖现有）
                                            if let Err(e) = self.import_todos(path) {
                                                eprintln!("导入任务失败: {}", e);
                                                self.show_confirm(
                                                    &format!("导入任务失败: {}", e),
                                                    ConfirmationAction::ImportTodos,
                                                );
                                            }
                                        } else if self.confirmation_message.contains("合并") || 
                                                 self.confirmation_message.contains("新任务") {
                                            // 合并导入
                                            if let Err(e) = self.merge_imported_todos(path) {
                                                eprintln!("合并导入任务失败: {}", e);
                                                self.show_confirm(
                                                    &format!("合并导入任务失败: {}", e),
                                                    ConfirmationAction::ImportTodos,
                                                );
                                            }
                                        }
                                        self.temp_input.clear();
                                    }
                                },
                                Some(ConfirmationAction::DeleteTag(_tag)) => {
                                    // 使用事先保存的标签名
                                    if !tag_to_delete.is_empty() {
                                        self.delete_tag(&tag_to_delete);
                                    }
                                },
                                None => {},
                            }
                            
                            self.show_confirmation = false;
                        }
                    });
                });
            });
    }

    /// 渲染Markdown预览器
    fn render_markdown_viewer(&mut self, ui: &mut Ui) {
        // 主要布局使用水平分割
        egui::SidePanel::left("markdown_sidebar")
            .resizable(true)
            .min_width(200.0)
            .max_width(400.0)
            .show_inside(ui, |ui| {
                // 左侧目录面板
                ui.vertical(|ui| {
                    ui.heading("文件目录");
                    ui.separator();
                    
                    // 添加导航按钮
                    ui.horizontal(|ui| {
                        if ui.button("返回").clicked() {
                            self.view = View::List;
                        }
                        
                        if ui.button("打开目录").clicked() {
                            if let Some(dir_path) = FileDialog::new()
                                .set_directory(".")
                                .pick_folder() {
                                
                                match markdown::get_markdown_files(&dir_path) {
                                    Ok(files) => {
                                        self.markdown_files = files;
                                        self.current_markdown_directory = Some(dir_path.to_string_lossy().to_string());
                                        
                                        // 清除当前文件内容
                                        self.current_markdown_path = None;
                                        self.markdown_content.clear();
                                    },
                                    Err(err) => {
                                        self.show_confirm(
                                            &format!("无法加载Markdown目录: {}", err),
                                            ConfirmationAction::ImportTodos,
                                        );
                                    }
                                }
                            }
                        }
                        
                        if ui.button("打开文件").clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("Markdown", &["md", "markdown"])
                                .set_directory(".")
                                .pick_file() {
                                
                                match markdown::load_markdown_file(&path) {
                                    Ok(content) => {
                                        self.markdown_content = content;
                                        self.current_markdown_path = Some(path.to_string_lossy().to_string());
                                        
                                        // 更新目录信息（如果文件在当前目录中）
                                        if let Some(parent) = path.parent() {
                                            if self.current_markdown_directory.is_none() {
                                                self.current_markdown_directory = Some(parent.to_string_lossy().to_string());
                                                // 加载目录中的其他文件
                                                if let Ok(files) = markdown::get_markdown_files(parent) {
                                                    self.markdown_files = files;
                                                }
                                            }
                                        }
                                    },
                                    Err(err) => {
                                        self.show_confirm(
                                            &format!("无法加载Markdown文件: {}", err),
                                            ConfirmationAction::ImportTodos,
                                        );
                                    }
                                }
                            }
                        }
                    });
                    
                    ui.separator();
                    
                    // 显示当前目录路径
                    if let Some(dir_path) = &self.current_markdown_directory {
                        ui.label(RichText::new(format!("目录: {}", dir_path)).italics());
                        ui.separator();
                        
                        // 文件列表
                        let md_files = self.markdown_files.clone();
                        let current_path = self.current_markdown_path.clone();
                        let dir_path_str = dir_path.clone();
                        let theme_accent = self.theme.accent;
                        
                        ScrollArea::vertical().show(ui, |ui| {
                            if md_files.is_empty() {
                                ui.label("此目录没有Markdown文件");
                            } else {
                                for file_name in &md_files {
                                    // 判断是否为当前选中的文件
                                    let is_selected = current_path.as_ref()
                                        .map(|path| path.ends_with(file_name))
                                        .unwrap_or(false);
                                    
                                    let text = if is_selected {
                                        RichText::new(file_name).strong().color(theme_accent)
                                    } else {
                                        RichText::new(file_name)
                                    };
                                    
                                    // 使用克隆的文件名
                                    let file_name_clone = file_name.clone();
                                    if ui.selectable_label(is_selected, text).clicked() {
                                        // 构建完整文件路径
                                        let full_path = std::path::Path::new(&dir_path_str).join(&file_name_clone);
                                        
                                        // 加载选中的文件
                                        match markdown::load_markdown_file(&full_path) {
                                            Ok(content) => {
                                                self.markdown_content = content;
                                                self.current_markdown_path = Some(full_path.to_string_lossy().to_string());
                                            },
                                            Err(err) => {
                                                let error_msg = format!("无法加载Markdown文件: {}", err);
                                                self.show_confirm(
                                                    &error_msg,
                                                    ConfirmationAction::ImportTodos,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("未选择目录");
                        });
                    }
                });
            });
            
        // 右侧内容预览面板
        egui::CentralPanel::default().show_inside(ui, |ui| {
            // 文件内容区域
            ui.vertical(|ui| {
                // 显示当前文件名
                if let Some(path) = &self.current_markdown_path {
                    let file_name = std::path::Path::new(path)
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| "未知文件".to_string());
                    
                    ui.heading(file_name);
                    ui.separator();
                    
                    // Markdown内容预览区域
                    let content = self.markdown_content.clone();
                    let is_dark_mode = ui.visuals().dark_mode;
                    
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            if !content.is_empty() {
                                // 设置宽度以填充可用空间
                                ui.set_width(ui.available_width());
                                
                                // 使用frame来给内容添加一些边距和背景
                                egui::Frame::none()
                                    .inner_margin(egui::Margin::same(16.0))
                                    .show(ui, |ui| {
                                        markdown::render_markdown(ui, &content, is_dark_mode);
                                    });
                            }
                        });
                } else {
                    // 没有选择文件时显示提示
                    ui.vertical_centered(|ui| {
                        ui.heading("Markdown预览");
                        ui.separator();
                        
                        ui.add_space(100.0);
                        
                        ui.label(RichText::new("请在左侧选择文件或点击\"打开文件\"按钮").size(18.0));
                        
                        ui.add_space(20.0);
                        
                        if ui.button("打开文件").clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("Markdown", &["md", "markdown"])
                                .set_directory(".")
                                .pick_file() {
                                
                                match markdown::load_markdown_file(&path) {
                                    Ok(content) => {
                                        self.markdown_content = content;
                                        self.current_markdown_path = Some(path.to_string_lossy().to_string());
                                        
                                        // 更新目录信息
                                        if let Some(parent) = path.parent() {
                                            self.current_markdown_directory = Some(parent.to_string_lossy().to_string());
                                            // 加载目录中的其他文件
                                            if let Ok(files) = markdown::get_markdown_files(parent) {
                                                self.markdown_files = files;
                                            }
                                        }
                                    },
                                    Err(err) => {
                                        self.show_confirm(
                                            &format!("无法加载Markdown文件: {}", err),
                                            ConfirmationAction::ImportTodos,
                                        );
                                    }
                                }
                            }
                        }
                    });
                }
            });
        });
    }
}
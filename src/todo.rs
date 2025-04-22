use chrono::{DateTime, Local, Datelike};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 表情符号类型，用于为每个任务添加视觉辨识度
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Emoji {
    None,
    CheckMark,
    Star,
    Heart,
    Fire,
    Book,
    Music,
    Sport,
    Shopping,
    Work,
    Family,
    Health,
    Travel,
    Custom(String),
}

impl Emoji {
    /// 获取表情符号的Unicode字符
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Emoji::None => "",
            Emoji::CheckMark => "✅",
            Emoji::Star => "⭐",
            Emoji::Heart => "❤️",
            Emoji::Fire => "🔥",
            Emoji::Book => "📚",
            Emoji::Music => "🎵",
            Emoji::Sport => "🏃",
            Emoji::Shopping => "🛒",
            Emoji::Work => "💼",
            Emoji::Family => "👪",
            Emoji::Health => "🏥",
            Emoji::Travel => "✈️",
            Emoji::Custom(s) => s,
        }
    }

    /// 获取所有预定义表情符号
    pub fn all_emojis() -> Vec<Emoji> {
        vec![
            Emoji::None,
            Emoji::CheckMark,
            Emoji::Star,
            Emoji::Heart,
            Emoji::Fire,
            Emoji::Book,
            Emoji::Music,
            Emoji::Sport,
            Emoji::Shopping,
            Emoji::Work,
            Emoji::Family,
            Emoji::Health,
            Emoji::Travel,
        ]
    }

    /// 获取随机表情符号
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let emojis = Self::all_emojis();
        emojis[1..].choose(&mut rng).unwrap().clone() // 跳过None选项
    }
}

/// 任务优先级
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// 获取优先级的字符表示
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
            Priority::Critical => "紧急",
        }
    }

    /// 获取优先级的颜色
    #[allow(dead_code)]
    pub fn color(&self) -> egui::Color32 {
        match self {
            Priority::Low => egui::Color32::from_rgb(76, 175, 80),     // 绿色
            Priority::Medium => egui::Color32::from_rgb(255, 193, 7),  // 黄色
            Priority::High => egui::Color32::from_rgb(255, 87, 34),    // 橙色
            Priority::Critical => egui::Color32::from_rgb(244, 67, 54), // 红色
        }
    }

    /// 所有优先级选项
    #[allow(dead_code)]
    pub fn all_priorities() -> Vec<Priority> {
        vec![
            Priority::Low,
            Priority::Medium,
            Priority::High,
            Priority::Critical,
        ]
    }
}

/// 单个待办事项
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub created_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
    pub due_date: Option<DateTime<Local>>,
    pub priority: Priority,
    pub emoji: Emoji,
    pub tags: Vec<String>,
    pub subtasks: Vec<SubTask>,
}

/// 子任务
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

impl PartialEq for SubTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && 
        self.title == other.title && 
        self.completed == other.completed
    }
}

impl SubTask {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            completed: false,
        }
    }
}

impl Todo {
    /// 创建新的待办事项
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description: String::new(),
            completed: false,
            created_at: Local::now(),
            completed_at: None,
            due_date: None,
            priority: Priority::Medium,
            emoji: Emoji::random(),
            tags: Vec::new(),
            subtasks: Vec::new(),
        }
    }

    /// 检查任务是否已过期
    #[allow(dead_code)]
    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            !self.completed && due < Local::now()
        } else {
            false
        }
    }

    /// 获取完成百分比
    #[allow(dead_code)]
    pub fn completion_percentage(&self) -> f32 {
        if self.subtasks.is_empty() {
            if self.completed {
                1.0
            } else {
                0.0
            }
        } else {
            let completed = self.subtasks.iter().filter(|t| t.completed).count();
            completed as f32 / self.subtasks.len() as f32
        }
    }

    /// 设置任务完成状态
    pub fn set_completed(&mut self, completed: bool) {
        // 如果状态变为已完成且之前不是已完成状态，记录完成时间
        if completed && !self.completed {
            self.completed_at = Some(Local::now());
        } else if !completed {
            // 如果标记为未完成，清除完成时间
            self.completed_at = None;
        }
        self.completed = completed;
    }

    /// 格式化日期时间为友好字符串
    pub fn format_date_time(dt: &DateTime<Local>) -> String {
        // 获取当前时间
        let now = Local::now();
        let duration = now.signed_duration_since(*dt);
        
        // 今天的日期
        let today = now.date_naive();
        let dt_date = dt.date_naive();
        
        if duration.num_seconds() < 60 {
            // 不到一分钟
            "刚刚".to_string()
        } else if duration.num_minutes() < 60 {
            // 不到一小时
            format!("{}分钟前", duration.num_minutes())
        } else if duration.num_hours() < 24 && dt_date == today {
            // 今天内
            format!("今天 {}", dt.format("%H:%M"))
        } else if (today - dt_date).num_days() == 1 {
            // 昨天
            format!("昨天 {}", dt.format("%H:%M"))
        } else if (today - dt_date).num_days() < 7 {
            // 一周内
            format!("{} {}", dt.format("%a"), dt.format("%H:%M"))
        } else if dt.year() == now.year() {
            // 今年内
            dt.format("%m-%d %H:%M").to_string()
        } else {
            // 更早
            dt.format("%Y-%m-%d %H:%M").to_string()
        }
    }
}

/// 待办事项列表
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoList {
    pub todos: HashMap<String, Todo>,
    pub active_tags: Vec<String>,
    pub filter_completed: bool,
    pub priority_sort: Option<bool>, // true表示从高到低排序，false表示从低到高，None表示默认按时间排序
}

impl Default for TodoList {
    fn default() -> Self {
        Self {
            todos: HashMap::new(),
            active_tags: Vec::new(),
            filter_completed: false,
            priority_sort: None, // 默认按时间排序
        }
    }
}

impl TodoList {
    /// 添加新待办事项
    #[allow(dead_code)]
    pub fn add_todo(&mut self, todo: Todo) {
        self.todos.insert(todo.id.clone(), todo);
    }

    /// 删除待办事项
    pub fn remove_todo(&mut self, id: &str) {
        self.todos.remove(id);
    }

    /// 获取所有标签
    #[allow(dead_code)]
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags = Vec::new();
        for todo in self.todos.values() {
            for tag in &todo.tags {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
        }
        tags.sort();
        tags
    }

    /// 获取过滤后的待办事项列表
    pub fn filtered_todos(&self) -> Vec<&Todo> {
        let mut result: Vec<&Todo> = self.todos.values().collect();
        
        // 按标签过滤
        if !self.active_tags.is_empty() {
            result = result
                .into_iter()
                .filter(|todo| {
                    self.active_tags
                        .iter()
                        .any(|tag| todo.tags.contains(tag))
                })
                .collect();
        }
        
        // 过滤已完成的任务
        if self.filter_completed {
            result = result.into_iter().filter(|todo| !todo.completed).collect();
        }
        
        // 按优先级和日期排序
        result.sort_by(|a, b| {
            // 先按完成状态
            let comp = a.completed.cmp(&b.completed);
            if comp != std::cmp::Ordering::Equal {
                return comp;
            }
            
            // 根据优先级排序设置进行排序
            if !a.completed {
                // 如果启用了优先级排序
                if let Some(high_to_low) = self.priority_sort {
                    let a_prio = priority_to_number(&a.priority);
                    let b_prio = priority_to_number(&b.priority);
                    
                    // 根据排序方向决定比较方式
                    let prio_comp = if high_to_low {
                        b_prio.cmp(&a_prio) // 高优先级在前
                    } else {
                        a_prio.cmp(&b_prio) // 低优先级在前
                    };
                    
                    if prio_comp != std::cmp::Ordering::Equal {
                        return prio_comp;
                    }
                }
            }
            
            // 默认按创建日期排序（从新到旧）
            b.created_at.cmp(&a.created_at)
        });
        
        result
    }

    /// 保存到文件
    pub fn save(&self) -> Result<(), String> {
        let app_dirs = directories::ProjectDirs::from("com", "rodo", "Rodo")
            .ok_or_else(|| "无法确定应用程序目录".to_string())?;
        
        let data_dir = app_dirs.data_dir();
        std::fs::create_dir_all(data_dir).map_err(|e| format!("无法创建数据目录: {}", e))?;
        
        let file_path = data_dir.join("todos.json");
        let serialized = serde_json::to_string(self).map_err(|e| format!("序列化失败: {}", e))?;
        
        std::fs::write(file_path, serialized).map_err(|e| format!("写入文件失败: {}", e))?;
        
        Ok(())
    }

    /// 从文件加载
    pub fn load() -> Self {
        let app_dirs = match directories::ProjectDirs::from("com", "rodo", "Rodo") {
            Some(dirs) => dirs,
            None => return Self::default(),
        };
        
        let data_dir = app_dirs.data_dir();
        let file_path = data_dir.join("todos.json");
        
        if !file_path.exists() {
            return Self::default();
        }
        
        let data = match std::fs::read_to_string(file_path) {
            Ok(data) => data,
            Err(_) => return Self::default(),
        };
        
        match serde_json::from_str(&data) {
            Ok(list) => list,
            Err(_) => Self::default(),
        }
    }

    /// 导出待办事项列表到指定文件
    pub fn export_to_file(&self, file_path: &std::path::Path) -> Result<(), String> {
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化失败: {}", e))?;
        
        std::fs::write(file_path, serialized)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        
        Ok(())
    }
    
    /// 从指定文件导入待办事项列表
    pub fn import_from_file(file_path: &std::path::Path) -> Result<Self, String> {
        let data = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        
        let todo_list: Self = serde_json::from_str(&data)
            .map_err(|e| format!("解析JSON失败: {}", e))?;
        
        Ok(todo_list)
    }
}

/// 将优先级转换为数字以便排序
fn priority_to_number(priority: &Priority) -> u8 {
    match priority {
        Priority::Low => 0,
        Priority::Medium => 1,
        Priority::High => 2,
        Priority::Critical => 3,
    }
}
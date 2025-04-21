use crate::theme::Theme;
use crate::todo::{Emoji, Priority, SubTask, Todo, TodoList};
use egui::FontId;

/// 应用程序的主视图部分
#[derive(Debug, PartialEq, Clone)]
pub enum View {
    /// 待办事项列表视图
    List,
    /// 添加新待办事项视图
    AddTodo,
    /// 编辑已有待办事项视图
    EditTodo,
    /// 设置视图
    Settings,
    /// 统计视图
    #[allow(dead_code)]
    Stats,
    /// 标签管理视图
    Tags,
    /// 关于视图
    #[allow(dead_code)]
    About,
}

/// 应用程序状态
pub struct RodoApp {
    /// 当前视图
    pub view: View,
    /// 任务列表
    pub todo_list: TodoList,
    /// 应用主题
    pub theme: Theme,
    /// 编辑中任务的ID
    pub editing_todo_id: Option<String>,
    /// 新任务（用于添加新任务）
    pub new_todo: Todo,
    /// 临时文本输入
    pub temp_input: String,
    /// 临时标签输入
    pub temp_tag_input: String,
    /// 是否已修改（用于保存）
    pub modified: bool,
    /// 显示确认对话框
    pub show_confirmation: bool,
    /// 确认对话框消息
    pub confirmation_message: String,
    /// 确认对话框回调
    pub confirmation_action: Option<ConfirmationAction>,
}

/// 确认对话框动作类型
#[derive(Debug, Clone)]
pub enum ConfirmationAction {
    DeleteTodo(String),
    #[allow(dead_code)]
    DeleteAllCompleted,
    #[allow(dead_code)]
    ResetSettings,
    ImportTodos,
    DeleteTag(String),
}

impl Default for RodoApp {
    fn default() -> Self {
        Self {
            view: View::List,
            todo_list: TodoList::load(),
            theme: Theme::default(),
            editing_todo_id: None,
            new_todo: Todo::new(String::new()),
            temp_input: String::new(),
            temp_tag_input: String::new(),
            modified: false,
            show_confirmation: false,
            confirmation_message: String::new(),
            confirmation_action: None,
        }
    }
}

impl RodoApp {
    /// 创建新的应用实例
    #[allow(dead_code)]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 获取上下文
        let ctx = &cc.egui_ctx;
        
        // 配置样式
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            FontId::new(24.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(16.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            FontId::new(13.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            FontId::new(16.0, egui::FontFamily::Proportional),
        );
        
        // 应用样式
        ctx.set_style(style);
        
        // 加载应用状态，包括任务和主题
        let todo_list = TodoList::load();
        let theme = Theme::load();
        
        let mut app = Self {
            view: View::List,
            todo_list,
            theme,
            editing_todo_id: None,
            new_todo: Todo::new(String::new()),
            temp_input: String::new(),
            temp_tag_input: String::new(),
            modified: false,
            show_confirmation: false,
            confirmation_message: String::new(),
            confirmation_action: None,
        };
        
        // 应用主题
        app.theme.apply_to_ctx(ctx);
        
        // 添加一些示例任务，如果没有任务的话
        if app.todo_list.todos.is_empty() {
            app.add_sample_todos();
        }
        
        app
    }
    
    /// 如果没有任务，添加一些示例任务
    #[allow(dead_code)]
    fn add_sample_todos(&mut self) {
        // 示例任务1：项目计划
        let mut todo1 = Todo::new("完成Rodo项目功能开发".to_string());
        todo1.description = "实现所有计划的功能并进行测试".to_string();
        todo1.emoji = Emoji::Work;
        todo1.priority = Priority::High;
        todo1.tags = vec!["工作".to_string(), "编程".to_string()];
        
        // 添加子任务
        todo1.subtasks.push(SubTask::new("设计用户界面".to_string()));
        todo1.subtasks.push(SubTask::new("实现任务管理功能".to_string()));
        todo1.subtasks.push(SubTask::new("添加主题支持".to_string()));
        todo1.subtasks.push(SubTask::new("编写文档".to_string()));
        
        // 示例任务2：购物清单
        let mut todo2 = Todo::new("购买生活用品".to_string());
        todo2.emoji = Emoji::Shopping;
        todo2.priority = Priority::Medium;
        todo2.tags = vec!["个人".to_string(), "购物".to_string()];
        
        // 添加子任务
        todo2.subtasks.push(SubTask::new("洗发水".to_string()));
        todo2.subtasks.push(SubTask::new("牙膏".to_string()));
        todo2.subtasks.push(SubTask::new("洗衣液".to_string()));
        
        // 示例任务3：阅读
        let mut todo3 = Todo::new("阅读《Rust编程》".to_string());
        todo3.emoji = Emoji::Book;
        todo3.priority = Priority::Low;
        todo3.tags = vec!["学习".to_string(), "编程".to_string()];
        
        // 示例任务4：健身
        let mut todo4 = Todo::new("每周健身计划".to_string());
        todo4.emoji = Emoji::Sport;
        todo4.priority = Priority::Medium;
        todo4.tags = vec!["健康".to_string(), "个人".to_string()];
        todo4.description = "保持每周至少锻炼3次，每次30分钟以上".to_string();
        
        // 添加到列表
        self.todo_list.add_todo(todo1);
        self.todo_list.add_todo(todo2);
        self.todo_list.add_todo(todo3);
        self.todo_list.add_todo(todo4);
    }
    
    /// 保存应用程序状态
    pub fn save(&mut self) {
        if self.modified {
            if let Err(err) = self.todo_list.save() {
                eprintln!("保存失败: {}", err);
            }
            self.modified = false;
        }
    }
    
    /// 显示确认对话框
    pub fn show_confirm(&mut self, message: &str, action: ConfirmationAction) {
        self.confirmation_message = message.to_string();
        self.confirmation_action = Some(action);
        self.show_confirmation = true;
    }
    
    /// 创建新的待办事项
    #[allow(dead_code)]
    pub fn create_new_todo(&mut self) {
        if !self.new_todo.title.trim().is_empty() {
            self.todo_list.add_todo(self.new_todo.clone());
            self.new_todo = Todo::new(String::new());
            self.modified = true;
            self.view = View::List;
        }
    }
    
    /// 删除待办事项
    pub fn delete_todo(&mut self, id: &str) {
        self.todo_list.remove_todo(id);
        self.modified = true;
        
        // 如果正在编辑的任务被删除，返回列表视图
        if let Some(editing_id) = &self.editing_todo_id {
            if editing_id == id {
                self.editing_todo_id = None;
                self.view = View::List;
            }
        }
    }
    
    /// 删除所有已完成的任务
    pub fn delete_all_completed(&mut self) {
        let completed_ids: Vec<String> = self.todo_list.todos.values()
            .filter(|todo| todo.completed)
            .map(|todo| todo.id.clone())
            .collect();
            
        for id in completed_ids {
            self.todo_list.remove_todo(&id);
        }
        
        self.modified = true;
    }
    
    /// 导出待办事项到文件
    pub fn export_todos(&self, file_path: &std::path::Path) -> Result<(), String> {
        self.todo_list.export_to_file(file_path)
    }
    
    /// 从文件导入待办事项
    pub fn import_todos(&mut self, file_path: &std::path::Path) -> Result<(), String> {
        let imported_list = TodoList::import_from_file(file_path)?;
        self.todo_list = imported_list;
        self.modified = true;
        Ok(())
    }
    
    /// 合并导入的待办事项（保留现有任务，添加新任务）
    pub fn merge_imported_todos(&mut self, file_path: &std::path::Path) -> Result<usize, String> {
        let imported_list = TodoList::import_from_file(file_path)?;
        
        let mut imported_count = 0;
        for (id, todo) in imported_list.todos {
            if !self.todo_list.todos.contains_key(&id) {
                self.todo_list.todos.insert(id, todo);
                imported_count += 1;
            }
        }
        
        if imported_count > 0 {
            self.modified = true;
        }
        
        Ok(imported_count)
    }
    
    /// 删除指定标签（从所有任务中）
    pub fn delete_tag(&mut self, tag_name: &str) {
        for todo in self.todo_list.todos.values_mut() {
            todo.tags.retain(|t| t != tag_name);
        }
        
        // 同时从活跃标签中移除
        self.todo_list.active_tags.retain(|t| t != tag_name);
        
        self.modified = true;
    }
    
    /// 设置主题并保存
    pub fn set_theme(&mut self, theme: Theme, ctx: &egui::Context) {
        self.theme = theme;
        self.theme.apply_to_ctx(ctx);
        // 尝试保存主题设置
        if let Err(err) = self.theme.save() {
            eprintln!("保存主题设置失败: {}", err);
        }
    }
} 
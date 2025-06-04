use crate::Result;
use crate::commands::{apply, plan};
use crate::modules::loader::{ModuleData, load_modules_from_directory};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    ModuleSelection,
    PlanView,
    Applying,
}

pub struct TuiApp {
    pub state: AppState,
    pub modules: Vec<ModuleData>,
    pub selected_modules: HashSet<String>,
    pub current_selection: usize,
    pub filter: String,
    pub plan: Option<Vec<String>>,
    pub should_quit: bool,
    pub scroll_offset: usize,
    pub apply_status: Arc<Mutex<ApplyStatus>>,
    pub modules_path: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct ApplyStatus {
    pub is_running: bool,
    pub current_action: String,
    pub completed_actions: usize,
    pub total_actions: usize,
    pub error: Option<String>,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp {
    pub fn new() -> Self {
        let modules_path =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Load modules from the current working directory
        let modules = load_modules_from_directory(&modules_path).unwrap_or_else(|_| {
            // Log to file if TUI logging is enabled, otherwise just return empty vec
            tracing::error!("Failed to load modules from current directory");
            Vec::new()
        });

        Self {
            state: AppState::ModuleSelection,
            modules,
            selected_modules: HashSet::new(),
            current_selection: 0,
            filter: String::new(),
            plan: None,
            should_quit: false,
            scroll_offset: 0,
            apply_status: Arc::new(Mutex::new(ApplyStatus {
                is_running: false,
                current_action: String::new(),
                completed_actions: 0,
                total_actions: 0,
                error: None,
            })),
            modules_path,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.state {
            AppState::ModuleSelection => self.handle_module_selection_key(key),
            AppState::PlanView => self.handle_plan_view_key(key),
            AppState::Applying => self.handle_applying_key(key),
        }
    }

    fn handle_module_selection_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.current_selection > 0 {
                    self.current_selection -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let filtered_count = self.filtered_modules().len();
                if self.current_selection < filtered_count.saturating_sub(1) {
                    self.current_selection += 1;
                }
            }
            KeyCode::Char(' ') => {
                let modules = self.filtered_modules();
                if let Some(module) = modules.get(self.current_selection) {
                    let module_name = module.name.clone();
                    if self.selected_modules.contains(&module_name) {
                        self.selected_modules.remove(&module_name);
                    } else {
                        self.selected_modules.insert(module_name);
                    }
                }
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Select all visible
                let module_names: Vec<String> = self
                    .filtered_modules()
                    .into_iter()
                    .map(|m| m.name.clone())
                    .collect();
                for name in module_names {
                    self.selected_modules.insert(name);
                }
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Deselect all
                self.selected_modules.clear();
            }
            KeyCode::Enter => {
                if !self.selected_modules.is_empty() {
                    self.generate_plan();
                    self.state = AppState::PlanView;
                }
            }
            KeyCode::Char('/') => {
                // TODO: Implement search mode
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.current_selection = 0;
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.current_selection = 0;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_plan_view_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = AppState::ModuleSelection;
            }
            KeyCode::Enter | KeyCode::Char('a') => {
                self.start_apply();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(plan) = &self.plan {
                    if self.scroll_offset < plan.len().saturating_sub(1) {
                        self.scroll_offset += 1;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn filtered_modules(&self) -> Vec<&ModuleData> {
        self.modules
            .iter()
            .filter(|m| {
                if self.filter.is_empty() {
                    true
                } else {
                    m.name.contains(&self.filter)
                        || m.description
                            .as_ref()
                            .map(|d| d.contains(&self.filter))
                            .unwrap_or(false)
                }
            })
            .collect()
    }

    fn generate_plan(&mut self) {
        let modules: Vec<String> = self.selected_modules.iter().cloned().collect();

        match plan::execute(Some(modules), Some(self.modules_path.clone()), None) {
            Ok(plan_result) => {
                let mut plan_lines = vec![];

                for module_id in &plan_result.ordered_modules {
                    if let Some(module) = plan_result.registry.get(module_id) {
                        plan_lines.push(format!("Module: {}", module_id));
                        for action in &module.actions {
                            plan_lines.push(format!("  - {}", action.action_type));
                        }
                    }
                }

                self.plan = Some(plan_lines);
                self.scroll_offset = 0;
            }
            Err(e) => {
                self.plan = Some(vec![format!("Error generating plan: {}", e)]);
            }
        }
    }

    fn start_apply(&mut self) {
        let modules: Vec<String> = self.selected_modules.iter().cloned().collect();
        let modules_path = self.modules_path.clone();
        let status = Arc::clone(&self.apply_status);

        // Reset status
        {
            let mut s = status.lock().unwrap();
            s.is_running = true;
            s.current_action = "Starting apply...".to_string();
            s.completed_actions = 0;
            s.total_actions = 0;
            s.error = None;
        }

        self.state = AppState::Applying;

        // Run apply in a separate thread
        thread::spawn(
            move || match apply::execute(Some(modules), Some(modules_path), 4, None) {
                Ok(_) => {
                    let mut s = status.lock().unwrap();
                    s.is_running = false;
                    s.current_action = "Apply completed successfully!".to_string();
                }
                Err(e) => {
                    let mut s = status.lock().unwrap();
                    s.is_running = false;
                    s.error = Some(format!("Apply failed: {}", e));
                }
            },
        );
    }

    fn handle_applying_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                let status = self.apply_status.lock().unwrap();
                if !status.is_running {
                    self.state = AppState::ModuleSelection;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

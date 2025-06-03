use crate::Result;
use crate::modules::loader::{ModuleData, load_modules_from_directory};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashSet;

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
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp {
    pub fn new() -> Self {
        // Load modules from the examples directory
        let modules = load_modules_from_directory("examples").unwrap_or_else(|e| {
            eprintln!("Failed to load modules: {}", e);
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
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.state {
            AppState::ModuleSelection => self.handle_module_selection_key(key),
            AppState::PlanView => self.handle_plan_view_key(key),
            AppState::Applying => Ok(()), // No interaction during apply
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
                self.state = AppState::Applying;
                // TODO: Actually apply the modules
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
        let mut plan = vec![];
        for module_name in &self.selected_modules {
            plan.push(format!("Install module: {}", module_name));
            // TODO: Generate actual actions
        }
        self.plan = Some(plan);
        self.scroll_offset = 0;
    }
}

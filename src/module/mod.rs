use crate::actions::{Action, ActionType};
use dhd_macros::{typescript_fn, typescript_impl, typescript_type};

#[typescript_type]
pub struct ModuleDefinition {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub actions: Vec<ActionType>,
}

pub struct Module {
    pub name: String,
    pub actions: Vec<Box<dyn Action>>,
}

#[typescript_type]
pub struct ModuleBuilder {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    dependencies: Vec<String>,
}

#[typescript_impl]
impl ModuleBuilder {
    pub fn new(name: String) -> Self {
        ModuleBuilder {
            name,
            description: None,
            tags: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    // Alternative method that accepts individual strings
    pub fn tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn depends_on(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn actions(self, actions: Vec<ActionType>) -> ModuleDefinition {
        ModuleDefinition {
            name: self.name,
            description: self.description,
            tags: self.tags,
            dependencies: self.dependencies,
            actions,
        }
    }
}

#[typescript_fn]
pub fn define_module(name: String) -> ModuleBuilder {
    ModuleBuilder::new(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{PackageInstall, package_install::package_install};

    #[test]
    fn test_module_builder_basic() {
        let module = define_module("test".to_string()).actions(vec![]);

        assert_eq!(module.name, "test");
        assert_eq!(module.description, None);
        assert_eq!(module.tags.len(), 0);
        assert_eq!(module.actions.len(), 0);
    }

    #[test]
    fn test_module_builder_with_description() {
        let module = define_module("test".to_string())
            .description("Test module".to_string())
            .actions(vec![]);

        assert_eq!(module.name, "test");
        assert_eq!(module.description, Some("Test module".to_string()));
        assert_eq!(module.actions.len(), 0);
    }

    #[test]
    fn test_module_builder_with_actions() {
        let action = package_install(PackageInstall {
            names: vec!["vim".to_string()],
            manager: None,
        });

        let module = define_module("test".to_string())
            .description("Install vim".to_string())
            .actions(vec![action]);

        assert_eq!(module.name, "test");
        assert_eq!(module.description, Some("Install vim".to_string()));
        assert_eq!(module.actions.len(), 1);
    }

    #[test]
    fn test_module_builder_fluent_api() {
        // Test that the fluent API returns the correct types
        let builder = define_module("test".to_string());
        let builder_with_desc = builder.description("desc".to_string());
        let _module: ModuleDefinition = builder_with_desc.actions(vec![]);
    }

    #[test]
    fn test_module_builder_with_tags() {
        let module = define_module("test".to_string())
            .description("Test module".to_string())
            .tags(vec!["development".to_string(), "testing".to_string()])
            .actions(vec![]);

        assert_eq!(module.name, "test");
        assert_eq!(module.description, Some("Test module".to_string()));
        assert_eq!(
            module.tags,
            vec!["development".to_string(), "testing".to_string()]
        );
        assert_eq!(module.actions.len(), 0);
    }

    #[test]
    fn test_module_builder_with_dependencies() {
        let module = define_module("niri".to_string())
            .description("Window manager".to_string())
            .depends_on(vec!["waybar".to_string(), "swaync".to_string()])
            .actions(vec![]);

        assert_eq!(module.name, "niri");
        assert_eq!(
            module.dependencies,
            vec!["waybar".to_string(), "swaync".to_string()]
        );
    }

}

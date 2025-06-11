use dhd_macros::{typescript_type, typescript_fn};
use super::{Action, ActionType};
use super::condition::Condition;
use std::path::Path;

/// A conditional wrapper action that executes a child action based on conditions
#[typescript_type]
pub struct ConditionalAction {
    /// The wrapped action to conditionally execute
    pub action: Box<ActionType>,
    /// Conditions to evaluate - if using multiple, they're AND'ed together
    pub conditions: Vec<Condition>,
    /// If true, skip when conditions pass (skipIf), if false run only when conditions pass (onlyIf)
    pub skip_on_success: bool,
}

impl ConditionalAction {
    pub fn skip_if(action: ActionType, conditions: Vec<Condition>) -> Self {
        Self {
            action: Box::new(action),
            conditions,
            skip_on_success: true,
        }
    }

    pub fn only_if(action: ActionType, conditions: Vec<Condition>) -> Self {
        Self {
            action: Box::new(action),
            conditions,
            skip_on_success: false,
        }
    }

    /// Evaluate all conditions and determine if the action should execute
    fn should_execute(&self) -> Result<bool, String> {
        // Evaluate all conditions (AND logic by default)
        for condition in &self.conditions {
            let passed = condition.evaluate()?;
            
            if !passed && !self.skip_on_success {
                // For onlyIf, if any condition fails, we don't execute
                return Ok(false);
            }
            
            if passed && self.skip_on_success {
                // For skipIf, if any condition passes, we skip
                return Ok(false);
            }
        }
        
        // All conditions evaluated
        // For skipIf: no conditions passed, so we execute (return true)
        // For onlyIf: all conditions passed, so we execute (return true)
        Ok(true)
    }
}

impl Action for ConditionalAction {
    fn name(&self) -> &str {
        self.action.name()
    }

    fn plan(&self, module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        match self.should_execute() {
            Ok(true) => {
                let condition_desc = if self.conditions.len() == 1 {
                    self.conditions[0].describe()
                } else {
                    format!("all of: [{}]", 
                        self.conditions.iter()
                            .map(|c| c.describe())
                            .collect::<Vec<_>>()
                            .join(", "))
                };
                println!("Condition '{}' allows execution, running action '{}'", condition_desc, self.name());
                self.action.plan(module_dir)
            },
            Ok(false) => {
                let condition_desc = if self.conditions.len() == 1 {
                    self.conditions[0].describe()
                } else {
                    format!("all of: [{}]", 
                        self.conditions.iter()
                            .map(|c| c.describe())
                            .collect::<Vec<_>>()
                            .join(", "))
                };
                let action_verb = if self.skip_on_success { "caused skip" } else { "not met" };
                println!("Condition '{}' {}, skipping action '{}'", condition_desc, action_verb, self.name());
                vec![]
            },
            Err(e) => {
                eprintln!("Error evaluating conditions for action '{}': {}", self.name(), e);
                vec![]
            }
        }
    }
}

#[typescript_fn]
/// Skip an action if the conditions pass
/// 
/// Example:
/// ```typescript
/// skipIf(
///     packageInstall({ names: ["nvidia-driver"] }),
///     [not(commandSucceeds("lspci | grep -q NVIDIA", null))]
/// )
/// ```
pub fn skip_if(action: ActionType, conditions: Vec<Condition>) -> ActionType {
    ActionType::Conditional(ConditionalAction::skip_if(action, conditions))
}

#[typescript_fn]
/// Only run an action if the conditions pass
/// 
/// Example:
/// ```typescript
/// onlyIf(
///     copyFile({ source: "fingerprint.conf", target: "/etc/pam.d/sudo", escalate: true }),
///     [anyOf([
///         commandSucceeds("lsusb | grep -qi fingerprint", null),
///         commandSucceeds("lshw -C biometric 2>/dev/null | grep -qi fingerprint", null)
///     ])]
/// )
/// ```
pub fn only_if(action: ActionType, conditions: Vec<Condition>) -> ActionType {
    ActionType::Conditional(ConditionalAction::only_if(action, conditions))
}
use crate::actions::Action;
use crate::{Atom, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    pub user: String,
    pub groups: Vec<String>,
    pub append: Option<bool>,
}

impl Action for UserGroup {
    fn plan(&self) -> Result<Vec<Box<dyn Atom>>> {
        let append = self.append.unwrap_or(true);

        // If user is "current", get the actual username
        let username = if self.user == "current" || self.user == "${USER}" {
            std::env::var("USER").unwrap_or_else(|_| whoami::username())
        } else {
            self.user.clone()
        };

        debug!(
            "Planning user group modification: user={}, groups={:?}, append={}",
            username, self.groups, append
        );

        let mut atoms: Vec<Box<dyn Atom>> = vec![];

        // Create the usermod command
        let mut args = vec![];

        if append {
            args.push("-a".to_string()); // Append to existing groups
        }

        args.push("-G".to_string());
        args.push(self.groups.join(","));
        args.push(username);

        // Use sudo to run usermod with elevated privileges
        atoms.push(Box::new(crate::atoms::RunCommand {
            command: "sudo".to_string(),
            args: Some([vec!["usermod".to_string()], args].concat()),
            cwd: None,
            env: None,
            shell: None,
        }));

        Ok(atoms)
    }

    fn describe(&self) -> String {
        format!(
            "Modify user '{}' groups: {}",
            self.user,
            self.groups.join(", ")
        )
    }
}

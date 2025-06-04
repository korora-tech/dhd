use std::process::Command;

/// Detects and returns the available privilege escalation command.
/// 
/// Checks for privilege escalation commands in order of preference:
/// 1. run0 (systemd-run --uid=0)
/// 2. doas
/// 3. sudo
/// 
/// Returns the first available command as a String, or None if no command is available.
pub fn detect_privilege_escalation_command() -> Option<String> {
    // Define commands in order of preference
    let commands = [
        "run0",     // Modern systemd-based privilege escalation
        "doas",     // OpenBSD-style, simpler alternative to sudo
        "sudo",     // Traditional privilege escalation
    ];

    for cmd in &commands {
        if command_exists(cmd) {
            return Some(cmd.to_string());
        }
    }

    None
}

/// Checks if a command exists in the system PATH
fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Returns the full command arguments for the detected privilege escalation tool.
/// 
/// This function returns a vector of arguments that should be used with the command.
/// For example:
/// - sudo: returns vec!["sudo"]
/// - doas: returns vec!["doas"]
/// - run0: returns vec!["run0"]
pub fn get_privilege_escalation_args() -> Option<Vec<String>> {
    detect_privilege_escalation_command().map(|cmd| vec![cmd])
}

/// Executes a command with privilege escalation.
/// 
/// # Arguments
/// * `command` - The command to execute
/// * `args` - Arguments to pass to the command
/// 
/// # Returns
/// Result containing the Command output or an error
pub fn execute_with_privilege_escalation(
    command: &str,
    args: &[&str],
) -> Result<std::process::Output, std::io::Error> {
    let priv_cmd = detect_privilege_escalation_command()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No privilege escalation command found (sudo, doas, or run0)",
            )
        })?;

    let mut cmd = Command::new(&priv_cmd);
    cmd.arg(command);
    for arg in args {
        cmd.arg(arg);
    }

    cmd.output()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_privilege_escalation_command() {
        // This test will pass if at least one privilege escalation command is available
        let result = detect_privilege_escalation_command();
        
        // We can't guarantee which command will be available in test environments,
        // but we can check that the function returns a valid result
        if let Some(cmd) = result {
            assert!(["run0", "doas", "sudo"].contains(&cmd.as_str()));
        }
    }

    #[test]
    fn test_command_exists() {
        // Test with a command that should exist on most systems
        assert!(command_exists("ls") || command_exists("dir"));
        
        // Test with a command that shouldn't exist
        assert!(!command_exists("this_command_definitely_does_not_exist_12345"));
    }

    #[test]
    fn test_get_privilege_escalation_args() {
        let args = get_privilege_escalation_args();
        
        if let Some(args_vec) = args {
            assert!(!args_vec.is_empty());
            let cmd = &args_vec[0];
            assert!(["run0", "doas", "sudo"].contains(&cmd.as_str()));
        }
    }
}
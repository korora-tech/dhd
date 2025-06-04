use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Resolves a path relative to the executable's location.
///
/// This function helps find resources that are distributed alongside the binary,
/// such as example modules or default configurations.
///
/// # Arguments
/// * `relative_path` - The path relative to the executable location
///
/// # Returns
/// The absolute path resolved from the executable's directory
pub fn resolve_path_relative_to_binary(relative_path: &str) -> Result<PathBuf, std::io::Error> {
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine executable directory",
        )
    })?;

    Ok(exe_dir.join(relative_path))
}

/// Resolves a modules directory path.
///
/// This function resolves the modules directory based on:
/// 1. If an absolute path is provided, use it directly
/// 2. Otherwise, resolve relative to the current working directory
///
/// # Arguments
/// * `path` - The path to the modules directory
///
/// # Returns
/// The resolved modules directory path
pub fn resolve_modules_directory(path: &str) -> Result<PathBuf, std::io::Error> {
    let path_buf = PathBuf::from(path);

    // If it's an absolute path, use it directly
    if path_buf.is_absolute() {
        if path_buf.exists() && path_buf.is_dir() {
            return Ok(path_buf);
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Modules directory not found at: {}", path_buf.display()),
            ));
        }
    }

    // Otherwise, resolve relative to current working directory
    let cwd = env::current_dir()?;
    let resolved = cwd.join(path);

    if resolved.exists() && resolved.is_dir() {
        Ok(resolved)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Modules directory not found at: {}", resolved.display()),
        ))
    }
}

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
        "run0", // Modern systemd-based privilege escalation
        "doas", // OpenBSD-style, simpler alternative to sudo
        "sudo", // Traditional privilege escalation
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
    let priv_cmd = detect_privilege_escalation_command().ok_or_else(|| {
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
        assert!(!command_exists(
            "this_command_definitely_does_not_exist_12345"
        ));
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

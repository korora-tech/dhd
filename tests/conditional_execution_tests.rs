use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_test_modules(dir: &std::path::Path) {
    // Module that should always execute (true condition)
    let always_module = r#"
export default defineModule("always-run")
    .description("Module that always runs")
    .when(commandExists("sh"))  // sh exists on all Unix systems
    .actions([
        executeCommand({ 
            command: "echo",
            args: ["Module executed"],
            escalate: false
        })
    ]);
"#;
    fs::write(dir.join("always.ts"), always_module).unwrap();

    // Module that should never execute (false condition)
    let never_module = r#"
export default defineModule("never-run")
    .description("Module that never runs")
    .when(commandExists("impossible_command_12345"))
    .actions([
        executeCommand({ 
            command: "echo",
            args: ["This should never print"],
            escalate: false
        })
    ]);
"#;
    fs::write(dir.join("never.ts"), never_module).unwrap();

    // Module with OR conditions
    let or_module = r#"
export default defineModule("or-condition")
    .description("Module with OR conditions")
    .when(
        or([
            commandExists("impossible_command_12345"),
            fileExists("/etc/passwd")  // This should exist
        ])
    )
    .actions([
        executeCommand({ 
            command: "echo",
            args: ["OR condition passed"],
            escalate: false
        })
    ]);
"#;
    fs::write(dir.join("or_condition.ts"), or_module).unwrap();

    // Module with AND conditions
    let and_module = r#"
export default defineModule("and-condition")
    .description("Module with AND conditions")
    .when(
        and([
            commandExists("sh"),
            fileExists("/etc/passwd")
        ])
    )
    .actions([
        executeCommand({ 
            command: "echo",
            args: ["AND condition passed"],
            escalate: false
        })
    ]);
"#;
    fs::write(dir.join("and_condition.ts"), and_module).unwrap();

    // Module with NOT condition
    let not_module = r#"
export default defineModule("not-condition")
    .description("Module with NOT condition")
    .when(not(commandExists("impossible_command_12345")))
    .actions([
        executeCommand({ 
            command: "echo",
            args: ["NOT condition passed"],
            escalate: false
        })
    ]);
"#;
    fs::write(dir.join("not_condition.ts"), not_module).unwrap();

    // Module without conditions (should always run)
    let no_condition_module = r#"
export default defineModule("no-condition")
    .description("Module without conditions")
    .actions([
        executeCommand({ 
            command: "echo",
            args: ["No condition module"],
            escalate: false
        })
    ]);
"#;
    fs::write(dir.join("no_condition.ts"), no_condition_module).unwrap();
}

#[test]
fn test_module_with_true_condition_executes() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_modules(temp_dir.path());

    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .arg("--module")
        .arg("always-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module that always runs"))
        .stdout(predicate::str::contains("Total atoms: 1"))
        .stdout(predicate::str::contains("Completed: 1"));
}

#[test]
fn test_module_with_false_condition_skipped() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_modules(temp_dir.path());

    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .arg("--module")
        .arg("never-run")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module skipped due to condition"))
        .stdout(predicate::str::contains("Total atoms: 0"));
}

#[test]
fn test_or_condition_passes_when_one_true() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_modules(temp_dir.path());

    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .arg("--module")
        .arg("or-condition")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module with OR conditions"))
        .stdout(predicate::str::contains("Total atoms: 1"))
        .stdout(predicate::str::contains("Completed: 1"));
}

#[test]
fn test_and_condition_passes_when_all_true() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_modules(temp_dir.path());

    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .arg("--module")
        .arg("and-condition")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module with AND conditions"))
        .stdout(predicate::str::contains("Total atoms: 1"))
        .stdout(predicate::str::contains("Completed: 1"));
}

#[test]
fn test_not_condition() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_modules(temp_dir.path());

    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .arg("--module")
        .arg("not-condition")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module with NOT condition"))
        .stdout(predicate::str::contains("Total atoms: 1"))
        .stdout(predicate::str::contains("Completed: 1"));
}

#[test]
fn test_module_without_condition_always_runs() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_modules(temp_dir.path());

    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .arg("--module")
        .arg("no-condition")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module without conditions"))
        .stdout(predicate::str::contains("Total atoms: 1"))
        .stdout(predicate::str::contains("Completed: 1"));
}

#[test]
fn test_verbose_mode_shows_condition_details() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_modules(temp_dir.path());

    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .arg("--verbose")
        .arg("--module")
        .arg("never-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module skipped due to condition"))
        .stdout(predicate::str::contains("command exists: impossible_command_12345"));
}

#[test]
fn test_dry_run_with_conditions() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_modules(temp_dir.path());

    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .arg("--dry-run")
        .arg("--module")
        .arg("always-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"))
        .stdout(predicate::str::contains("Module that always runs"));
}

#[test]
fn test_multiple_modules_with_mixed_conditions() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_modules(temp_dir.path());

    // Create a module with complex conditions
    let complex_module = r#"
export default defineModule("complex-condition")
    .description("Complex condition module")
    .when(
        or([
            and([
                commandExists("sh"),
                not(commandExists("impossible_cmd"))
            ]),
            fileExists("/this/does/not/exist")
        ])
    )
    .actions([
        executeCommand({ 
            command: "echo",
            args: ["Complex condition passed"],
            escalate: false
        })
    ]);
"#;
    fs::write(temp_dir.path().join("complex.ts"), complex_module).unwrap();

    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .arg("--module")
        .arg("complex-condition")
        .assert()
        .success()
        .stdout(predicate::str::contains("Complex condition module"))
        .stdout(predicate::str::contains("Total atoms: 1"));
}

// TODO: Enable this test when action-level conditions are implemented
// #[test]
// fn test_action_level_conditions() {
//     let temp_dir = TempDir::new().unwrap();
//     
//     // Module with conditional actions
//     let conditional_actions_module = r#"
// export default defineModule("conditional-actions")
//     .description("Module with conditional actions")
//     .actions([
//         executeCommand({ 
//             command: "echo",
//             args: ["Always runs"],
//             escalate: false
//         }),
//         onlyIf(
//             executeCommand({ 
//                 command: "echo",
//                 args: ["Runs if /etc/passwd exists"],
//                 escalate: false
//             }),
//             [fileExists("/etc/passwd")]
//         ),
//         skipIf(
//             executeCommand({ 
//                 command: "echo",
//                 args: ["Should be skipped"],
//                 escalate: false
//             }),
//             [commandExists("sh")]  // This exists, so action should be skipped
//         )
//     ]);
// "#;
//     fs::write(temp_dir.path().join("conditional_actions.ts"), conditional_actions_module).unwrap();
// 
//     let mut cmd = Command::cargo_bin("dhd").unwrap();
//     cmd.current_dir(temp_dir.path())
//         .arg("apply")
//         .arg("--verbose")
//         .arg("--module")
//         .arg("conditional-actions")
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Always runs"))
//         .stdout(predicate::str::contains("Runs if /etc/passwd exists"))
//         .stdout(predicate::str::contains("Total atoms: 2"));  // Only 2 should run, not 3
// }
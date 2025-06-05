use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemdServiceContent {
    pub unit: Option<UnitConfig>,
    pub service: ServiceConfig,
    pub install: Option<InstallConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemdSocketContent {
    pub unit: Option<UnitConfig>,
    pub socket: SocketConfig,
    pub install: Option<InstallConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnitConfig {
    pub description: Option<String>,
    pub documentation: Option<Vec<String>>,
    pub requires: Option<Vec<String>>,
    pub wants: Option<Vec<String>>,
    pub before: Option<Vec<String>>,
    pub after: Option<Vec<String>>,
    pub conflicts: Option<Vec<String>>,
    pub condition: Option<ConditionConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConditionConfig {
    pub path_exists: Option<String>,
    pub path_is_directory: Option<String>,
    pub file_not_empty: Option<String>,
    pub user: Option<String>,
    pub group: Option<String>,
    pub host: Option<String>,
    pub kernel_command_line: Option<String>,
    pub virtualization: Option<String>,
    pub architecture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServiceConfig {
    #[serde(rename = "type")]
    pub service_type: Option<String>,
    pub exec_start: Vec<String>,
    pub exec_start_pre: Option<Vec<String>>,
    pub exec_start_post: Option<Vec<String>>,
    pub exec_stop: Option<Vec<String>>,
    pub exec_stop_post: Option<Vec<String>>,
    pub exec_reload: Option<Vec<String>>,
    pub restart: Option<String>,
    pub restart_sec: Option<u32>,
    pub timeout_start_sec: Option<u32>,
    pub timeout_stop_sec: Option<u32>,
    pub timeout_sec: Option<u32>,
    pub remain_after_exit: Option<bool>,
    pub pid_file: Option<String>,
    pub bus_name: Option<String>,
    pub notify_access: Option<String>,
    pub watchdog_sec: Option<u32>,
    pub user: Option<String>,
    pub group: Option<String>,
    pub working_directory: Option<String>,
    pub root_directory: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    pub environment_file: Option<Vec<String>>,
    pub pass_environment: Option<Vec<String>>,
    pub unset_environment: Option<Vec<String>>,
    pub standard_input: Option<String>,
    pub standard_output: Option<String>,
    pub standard_error: Option<String>,
    pub tty_path: Option<String>,
    pub syslog_identifier: Option<String>,
    pub syslog_facility: Option<String>,
    pub syslog_level: Option<String>,
    pub syslog_level_prefix: Option<bool>,
    pub limit_cpu: Option<String>,
    pub limit_fsize: Option<String>,
    pub limit_data: Option<String>,
    pub limit_stack: Option<String>,
    pub limit_core: Option<String>,
    pub limit_rss: Option<String>,
    pub limit_nofile: Option<String>,
    pub limit_as: Option<String>,
    pub limit_nproc: Option<String>,
    pub limit_memlock: Option<String>,
    pub limit_locks: Option<String>,
    pub limit_sigpending: Option<String>,
    pub limit_msgqueue: Option<String>,
    pub limit_nice: Option<String>,
    pub limit_rtprio: Option<String>,
    pub limit_rttime: Option<String>,
    pub umask: Option<String>,
    pub nice: Option<i32>,
    pub oom_score_adjust: Option<i32>,
    pub io_scheduling_class: Option<String>,
    pub io_scheduling_priority: Option<u8>,
    pub cpu_scheduling_policy: Option<String>,
    pub cpu_scheduling_priority: Option<u8>,
    pub cpu_scheduling_reset_on_fork: Option<bool>,
    pub cpu_affinity: Option<String>,
    pub kill_mode: Option<String>,
    pub kill_signal: Option<String>,
    pub send_sigkill: Option<bool>,
    pub send_sighup: Option<bool>,
    pub private_tmp: Option<bool>,
    pub private_devices: Option<bool>,
    pub private_network: Option<bool>,
    pub private_users: Option<bool>,
    pub protect_system: Option<String>,
    pub protect_home: Option<String>,
    pub protect_kernel_tunables: Option<bool>,
    pub protect_kernel_modules: Option<bool>,
    pub protect_control_groups: Option<bool>,
    pub mount_flags: Option<String>,
    pub memory_deny_write_execute: Option<bool>,
    pub restrict_realtime: Option<bool>,
    pub restrict_suidsgid: Option<bool>,
    pub lock_personality: Option<bool>,
    pub no_new_privileges: Option<bool>,
    pub dynamic_user: Option<bool>,
    pub remove_ipc: Option<bool>,
    pub system_call_filter: Option<Vec<String>>,
    pub system_call_error_number: Option<String>,
    pub system_call_architectures: Option<Vec<String>>,
    pub restrict_address_families: Option<Vec<String>>,
    pub restrict_namespaces: Option<Vec<String>>,
    pub read_write_paths: Option<Vec<String>>,
    pub read_only_paths: Option<Vec<String>>,
    pub inaccessible_paths: Option<Vec<String>>,
    pub exec_paths: Option<Vec<String>>,
    pub no_exec_paths: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SocketConfig {
    pub listen_stream: Option<Vec<String>>,
    pub listen_datagram: Option<Vec<String>>,
    pub listen_sequential_packet: Option<Vec<String>>,
    pub listen_fifo: Option<Vec<String>>,
    pub listen_special: Option<Vec<String>>,
    pub listen_netlink: Option<Vec<String>>,
    pub listen_message_queue: Option<Vec<String>>,
    pub listen_usb_function: Option<Vec<String>>,
    pub bind_ipv6_only: Option<String>,
    pub backlog: Option<u32>,
    pub bind_to_device: Option<String>,
    pub socket_user: Option<String>,
    pub socket_group: Option<String>,
    pub socket_mode: Option<String>,
    pub directory_mode: Option<String>,
    pub accept: Option<bool>,
    pub writable: Option<bool>,
    pub max_connections: Option<u32>,
    pub max_connections_per_source: Option<u32>,
    pub keep_alive: Option<bool>,
    pub keep_alive_time_sec: Option<u32>,
    pub keep_alive_interval_sec: Option<u32>,
    pub keep_alive_probes: Option<u32>,
    pub no_delay: Option<bool>,
    pub priority: Option<i32>,
    pub defer_accept_sec: Option<u32>,
    pub receive_buffer: Option<String>,
    pub send_buffer: Option<String>,
    pub iptos: Option<u32>,
    pub ipttl: Option<u32>,
    pub mark: Option<u32>,
    pub reuse_port: Option<bool>,
    pub smack_label: Option<String>,
    pub smack_label_ipin: Option<String>,
    pub smack_label_ipout: Option<String>,
    pub selinux_context_from_net: Option<bool>,
    pub pipe_size: Option<String>,
    pub message_queue_max_messages: Option<u64>,
    pub message_queue_message_size: Option<u64>,
    pub free_bind: Option<bool>,
    pub transparent: Option<bool>,
    pub broadcast: Option<bool>,
    pub pass_credentials: Option<bool>,
    pub pass_security: Option<bool>,
    pub tcp_congestion: Option<String>,
    pub exec_start_pre: Option<Vec<String>>,
    pub exec_start_post: Option<Vec<String>>,
    pub exec_stop_pre: Option<Vec<String>>,
    pub exec_stop_post: Option<Vec<String>>,
    pub timeout_sec: Option<u32>,
    pub service: Option<String>,
    pub remove_on_stop: Option<bool>,
    pub symlinks: Option<Vec<String>>,
    pub file_descriptor_name: Option<String>,
    pub trigger_limit_interval_sec: Option<u32>,
    pub trigger_limit_burst: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InstallConfig {
    pub wanted_by: Option<Vec<String>>,
    pub required_by: Option<Vec<String>>,
    pub alias: Option<Vec<String>>,
    pub also: Option<Vec<String>>,
    pub default_instance: Option<String>,
}

pub fn build_service_unit(content: &SystemdServiceContent) -> String {
    let mut unit_file = String::new();

    // Build [Unit] section
    if let Some(unit) = &content.unit {
        unit_file.push_str("[Unit]\n");
        if let Some(desc) = &unit.description {
            unit_file.push_str(&format!("Description={}\n", desc));
        }
        if let Some(docs) = &unit.documentation {
            for doc in docs {
                unit_file.push_str(&format!("Documentation={}\n", doc));
            }
        }
        if let Some(requires) = &unit.requires {
            for req in requires {
                unit_file.push_str(&format!("Requires={}\n", req));
            }
        }
        if let Some(wants) = &unit.wants {
            for want in wants {
                unit_file.push_str(&format!("Wants={}\n", want));
            }
        }
        if let Some(before) = &unit.before {
            for b in before {
                unit_file.push_str(&format!("Before={}\n", b));
            }
        }
        if let Some(after) = &unit.after {
            for a in after {
                unit_file.push_str(&format!("After={}\n", a));
            }
        }
        if let Some(conflicts) = &unit.conflicts {
            for conflict in conflicts {
                unit_file.push_str(&format!("Conflicts={}\n", conflict));
            }
        }
        if let Some(condition) = &unit.condition {
            if let Some(path_exists) = &condition.path_exists {
                unit_file.push_str(&format!("ConditionPathExists={}\n", path_exists));
            }
            if let Some(path_is_dir) = &condition.path_is_directory {
                unit_file.push_str(&format!("ConditionPathIsDirectory={}\n", path_is_dir));
            }
            if let Some(file_not_empty) = &condition.file_not_empty {
                unit_file.push_str(&format!("ConditionFileNotEmpty={}\n", file_not_empty));
            }
            if let Some(user) = &condition.user {
                unit_file.push_str(&format!("ConditionUser={}\n", user));
            }
            if let Some(group) = &condition.group {
                unit_file.push_str(&format!("ConditionGroup={}\n", group));
            }
            if let Some(host) = &condition.host {
                unit_file.push_str(&format!("ConditionHost={}\n", host));
            }
            if let Some(kernel_cmd) = &condition.kernel_command_line {
                unit_file.push_str(&format!("ConditionKernelCommandLine={}\n", kernel_cmd));
            }
            if let Some(virt) = &condition.virtualization {
                unit_file.push_str(&format!("ConditionVirtualization={}\n", virt));
            }
            if let Some(arch) = &condition.architecture {
                unit_file.push_str(&format!("ConditionArchitecture={}\n", arch));
            }
        }
        unit_file.push('\n');
    }

    // Build [Service] section
    unit_file.push_str("[Service]\n");

    if let Some(service_type) = &content.service.service_type {
        unit_file.push_str(&format!("Type={}\n", service_type));
    }

    for exec in &content.service.exec_start {
        unit_file.push_str(&format!("ExecStart={}\n", exec));
    }

    if let Some(exec_pre) = &content.service.exec_start_pre {
        for exec in exec_pre {
            unit_file.push_str(&format!("ExecStartPre={}\n", exec));
        }
    }

    if let Some(exec_post) = &content.service.exec_start_post {
        for exec in exec_post {
            unit_file.push_str(&format!("ExecStartPost={}\n", exec));
        }
    }

    if let Some(exec_stop) = &content.service.exec_stop {
        for exec in exec_stop {
            unit_file.push_str(&format!("ExecStop={}\n", exec));
        }
    }

    if let Some(exec_post) = &content.service.exec_stop_post {
        for exec in exec_post {
            unit_file.push_str(&format!("ExecStopPost={}\n", exec));
        }
    }

    if let Some(exec_reload) = &content.service.exec_reload {
        for exec in exec_reload {
            unit_file.push_str(&format!("ExecReload={}\n", exec));
        }
    }

    if let Some(restart) = &content.service.restart {
        unit_file.push_str(&format!("Restart={}\n", restart));
    }

    if let Some(restart_sec) = content.service.restart_sec {
        unit_file.push_str(&format!("RestartSec={}\n", restart_sec));
    }

    if let Some(timeout) = content.service.timeout_start_sec {
        unit_file.push_str(&format!("TimeoutStartSec={}\n", timeout));
    }

    if let Some(timeout) = content.service.timeout_stop_sec {
        unit_file.push_str(&format!("TimeoutStopSec={}\n", timeout));
    }

    if let Some(timeout) = content.service.timeout_sec {
        unit_file.push_str(&format!("TimeoutSec={}\n", timeout));
    }

    if let Some(remain) = content.service.remain_after_exit {
        unit_file.push_str(&format!(
            "RemainAfterExit={}\n",
            if remain { "yes" } else { "no" }
        ));
    }

    if let Some(pid_file) = &content.service.pid_file {
        unit_file.push_str(&format!("PIDFile={}\n", pid_file));
    }

    if let Some(bus_name) = &content.service.bus_name {
        unit_file.push_str(&format!("BusName={}\n", bus_name));
    }

    if let Some(notify) = &content.service.notify_access {
        unit_file.push_str(&format!("NotifyAccess={}\n", notify));
    }

    if let Some(watchdog) = content.service.watchdog_sec {
        unit_file.push_str(&format!("WatchdogSec={}\n", watchdog));
    }

    if let Some(user) = &content.service.user {
        unit_file.push_str(&format!("User={}\n", user));
    }

    if let Some(group) = &content.service.group {
        unit_file.push_str(&format!("Group={}\n", group));
    }

    if let Some(work_dir) = &content.service.working_directory {
        unit_file.push_str(&format!("WorkingDirectory={}\n", work_dir));
    }

    if let Some(root_dir) = &content.service.root_directory {
        unit_file.push_str(&format!("RootDirectory={}\n", root_dir));
    }

    if let Some(env_map) = &content.service.environment {
        for (key, value) in env_map {
            unit_file.push_str(&format!("Environment=\"{}={}\"\n", key, value));
        }
    }

    if let Some(env_files) = &content.service.environment_file {
        for file in env_files {
            unit_file.push_str(&format!("EnvironmentFile={}\n", file));
        }
    }

    if let Some(pass_env) = &content.service.pass_environment {
        for env in pass_env {
            unit_file.push_str(&format!("PassEnvironment={}\n", env));
        }
    }

    if let Some(unset_env) = &content.service.unset_environment {
        for env in unset_env {
            unit_file.push_str(&format!("UnsetEnvironment={}\n", env));
        }
    }

    if let Some(stdin) = &content.service.standard_input {
        unit_file.push_str(&format!("StandardInput={}\n", stdin));
    }

    if let Some(stdout) = &content.service.standard_output {
        unit_file.push_str(&format!("StandardOutput={}\n", stdout));
    }

    if let Some(stderr) = &content.service.standard_error {
        unit_file.push_str(&format!("StandardError={}\n", stderr));
    }

    if let Some(private_tmp) = content.service.private_tmp {
        unit_file.push_str(&format!(
            "PrivateTmp={}\n",
            if private_tmp { "yes" } else { "no" }
        ));
    }

    if let Some(private_devices) = content.service.private_devices {
        unit_file.push_str(&format!(
            "PrivateDevices={}\n",
            if private_devices { "yes" } else { "no" }
        ));
    }

    if let Some(private_network) = content.service.private_network {
        unit_file.push_str(&format!(
            "PrivateNetwork={}\n",
            if private_network { "yes" } else { "no" }
        ));
    }

    if let Some(protect_system) = &content.service.protect_system {
        unit_file.push_str(&format!("ProtectSystem={}\n", protect_system));
    }

    if let Some(protect_home) = &content.service.protect_home {
        unit_file.push_str(&format!("ProtectHome={}\n", protect_home));
    }

    if let Some(no_new_privs) = content.service.no_new_privileges {
        unit_file.push_str(&format!(
            "NoNewPrivileges={}\n",
            if no_new_privs { "yes" } else { "no" }
        ));
    }

    unit_file.push('\n');

    // Build [Install] section
    if let Some(install) = &content.install {
        unit_file.push_str("[Install]\n");
        if let Some(wanted_by) = &install.wanted_by {
            for want in wanted_by {
                unit_file.push_str(&format!("WantedBy={}\n", want));
            }
        }
        if let Some(required_by) = &install.required_by {
            for req in required_by {
                unit_file.push_str(&format!("RequiredBy={}\n", req));
            }
        }
        if let Some(alias) = &install.alias {
            for a in alias {
                unit_file.push_str(&format!("Alias={}\n", a));
            }
        }
        if let Some(also) = &install.also {
            for a in also {
                unit_file.push_str(&format!("Also={}\n", a));
            }
        }
        if let Some(default) = &install.default_instance {
            unit_file.push_str(&format!("DefaultInstance={}\n", default));
        }
    }

    unit_file
}

pub fn build_socket_unit(content: &SystemdSocketContent) -> String {
    let mut unit_file = String::new();

    // Build [Unit] section
    if let Some(unit) = &content.unit {
        unit_file.push_str("[Unit]\n");
        if let Some(desc) = &unit.description {
            unit_file.push_str(&format!("Description={}\n", desc));
        }
        if let Some(docs) = &unit.documentation {
            for doc in docs {
                unit_file.push_str(&format!("Documentation={}\n", doc));
            }
        }
        if let Some(before) = &unit.before {
            for b in before {
                unit_file.push_str(&format!("Before={}\n", b));
            }
        }
        if let Some(after) = &unit.after {
            for a in after {
                unit_file.push_str(&format!("After={}\n", a));
            }
        }
        unit_file.push('\n');
    }

    // Build [Socket] section
    unit_file.push_str("[Socket]\n");

    if let Some(streams) = &content.socket.listen_stream {
        for stream in streams {
            unit_file.push_str(&format!("ListenStream={}\n", stream));
        }
    }

    if let Some(datagrams) = &content.socket.listen_datagram {
        for datagram in datagrams {
            unit_file.push_str(&format!("ListenDatagram={}\n", datagram));
        }
    }

    if let Some(fifos) = &content.socket.listen_fifo {
        for fifo in fifos {
            unit_file.push_str(&format!("ListenFIFO={}\n", fifo));
        }
    }

    if let Some(socket_user) = &content.socket.socket_user {
        unit_file.push_str(&format!("SocketUser={}\n", socket_user));
    }

    if let Some(socket_group) = &content.socket.socket_group {
        unit_file.push_str(&format!("SocketGroup={}\n", socket_group));
    }

    if let Some(socket_mode) = &content.socket.socket_mode {
        unit_file.push_str(&format!("SocketMode={}\n", socket_mode));
    }

    if let Some(accept) = content.socket.accept {
        unit_file.push_str(&format!("Accept={}\n", if accept { "yes" } else { "no" }));
    }

    if let Some(service) = &content.socket.service {
        unit_file.push_str(&format!("Service={}\n", service));
    }

    unit_file.push('\n');

    // Build [Install] section
    if let Some(install) = &content.install {
        unit_file.push_str("[Install]\n");
        if let Some(wanted_by) = &install.wanted_by {
            for want in wanted_by {
                unit_file.push_str(&format!("WantedBy={}\n", want));
            }
        }
        if let Some(required_by) = &install.required_by {
            for req in required_by {
                unit_file.push_str(&format!("RequiredBy={}\n", req));
            }
        }
    }

    unit_file
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_service() {
        let content = SystemdServiceContent {
            unit: Some(UnitConfig {
                description: Some("Test Service".to_string()),
                after: Some(vec!["network.target".to_string()]),
                ..Default::default()
            }),
            service: ServiceConfig {
                service_type: Some("simple".to_string()),
                exec_start: vec!["/usr/bin/test".to_string()],
                restart: Some("always".to_string()),
                ..Default::default()
            },
            install: Some(InstallConfig {
                wanted_by: Some(vec!["multi-user.target".to_string()]),
                ..Default::default()
            }),
        };

        let result = build_service_unit(&content);
        assert!(result.contains("[Unit]"));
        assert!(result.contains("Description=Test Service"));
        assert!(result.contains("After=network.target"));
        assert!(result.contains("[Service]"));
        assert!(result.contains("Type=simple"));
        assert!(result.contains("ExecStart=/usr/bin/test"));
        assert!(result.contains("Restart=always"));
        assert!(result.contains("[Install]"));
        assert!(result.contains("WantedBy=multi-user.target"));
    }

    #[test]
    fn test_build_simple_socket() {
        let content = SystemdSocketContent {
            unit: Some(UnitConfig {
                description: Some("Test Socket".to_string()),
                ..Default::default()
            }),
            socket: SocketConfig {
                listen_stream: Some(vec!["8080".to_string()]),
                accept: Some(false),
                ..Default::default()
            },
            install: Some(InstallConfig {
                wanted_by: Some(vec!["sockets.target".to_string()]),
                ..Default::default()
            }),
        };

        let result = build_socket_unit(&content);
        assert!(result.contains("[Unit]"));
        assert!(result.contains("Description=Test Socket"));
        assert!(result.contains("[Socket]"));
        assert!(result.contains("ListenStream=8080"));
        assert!(result.contains("Accept=no"));
        assert!(result.contains("[Install]"));
        assert!(result.contains("WantedBy=sockets.target"));
    }
}

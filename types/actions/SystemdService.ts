import type { BaseAction } from "./base";

// Systemd service types
export type ServiceType = "simple" | "exec" | "forking" | "oneshot" | "dbus" | "notify" | "idle";
export type RestartPolicy = "no" | "always" | "on-success" | "on-failure" | "on-abnormal" | "on-abort" | "on-watchdog";

// Environment configuration
export interface Environment {
	[key: string]: string;
}

// Unit section configuration
export interface UnitConfig {
	description?: string;
	documentation?: string | string[];
	requires?: string | string[];
	wants?: string | string[];
	before?: string | string[];
	after?: string | string[];
	conflicts?: string | string[];
	condition?: {
		pathExists?: string;
		pathIsDirectory?: string;
		fileNotEmpty?: string;
		user?: string;
		group?: string;
		host?: string;
		kernelCommandLine?: string;
		virtualization?: string | boolean;
		architecture?: string;
	};
}

// Service section configuration
export interface ServiceConfig {
	type?: ServiceType;
	execStart: string | string[];
	execStartPre?: string | string[];
	execStartPost?: string | string[];
	execStop?: string | string[];
	execStopPost?: string | string[];
	execReload?: string | string[];
	restart?: RestartPolicy;
	restartSec?: number;
	timeoutStartSec?: number;
	timeoutStopSec?: number;
	timeoutSec?: number;
	remainAfterExit?: boolean;
	pidFile?: string;
	busName?: string;
	notifyAccess?: "none" | "main" | "exec" | "all";
	watchdogSec?: number;
	user?: string;
	group?: string;
	workingDirectory?: string;
	rootDirectory?: string;
	environment?: Environment;
	environmentFile?: string | string[];
	passEnvironment?: string | string[];
	unsetEnvironment?: string | string[];
	standardInput?: "null" | "tty" | "tty-force" | "tty-fail" | "data" | "file" | "socket";
	standardOutput?: "inherit" | "null" | "tty" | "journal" | "kmsg" | "journal+console" | "kmsg+console" | "file" | "append" | "socket";
	standardError?: "inherit" | "null" | "tty" | "journal" | "kmsg" | "journal+console" | "kmsg+console" | "file" | "append" | "socket";
	ttyPath?: string;
	syslogIdentifier?: string;
	syslogFacility?: string;
	syslogLevel?: string;
	syslogLevelPrefix?: boolean;
	limitCPU?: string;
	limitFSIZE?: string;
	limitDATA?: string;
	limitSTACK?: string;
	limitCORE?: string;
	limitRSS?: string;
	limitNOFILE?: string;
	limitAS?: string;
	limitNPROC?: string;
	limitMEMLOCK?: string;
	limitLOCKS?: string;
	limitSIGPENDING?: string;
	limitMSGQUEUE?: string;
	limitNICE?: string;
	limitRTPRIO?: string;
	limitRTTIME?: string;
	umask?: string;
	nice?: number;
	oomScoreAdjust?: number;
	ioSchedulingClass?: "none" | "realtime" | "best-effort" | "idle";
	ioSchedulingPriority?: number;
	cpuSchedulingPolicy?: "other" | "batch" | "idle" | "fifo" | "rr";
	cpuSchedulingPriority?: number;
	cpuSchedulingResetOnFork?: boolean;
	cpuAffinity?: string;
	killMode?: "control-group" | "mixed" | "process" | "none";
	killSignal?: string;
	sendSIGKILL?: boolean;
	sendSIGHUP?: boolean;
	privateTmp?: boolean;
	privateDevices?: boolean;
	privateNetwork?: boolean;
	privateUsers?: boolean;
	protectSystem?: boolean | "full" | "strict";
	protectHome?: boolean | "read-only" | "tmpfs";
	protectKernelTunables?: boolean;
	protectKernelModules?: boolean;
	protectControlGroups?: boolean;
	mountFlags?: "shared" | "slave" | "private";
	memoryDenyWriteExecute?: boolean;
	restrictRealtime?: boolean;
	restrictSUIDSGID?: boolean;
	lockPersonality?: boolean;
	noNewPrivileges?: boolean;
	dynamicUser?: boolean;
	removeipc?: boolean;
	systemCallFilter?: string | string[];
	systemCallErrorNumber?: string;
	systemCallArchitectures?: string | string[];
	restrictAddressFamilies?: string | string[];
	restrictNamespaces?: string | string[];
	readWritePaths?: string | string[];
	readOnlyPaths?: string | string[];
	inaccessiblePaths?: string | string[];
	execPaths?: string | string[];
	noExecPaths?: string | string[];
}

// Install section configuration
export interface InstallConfig {
	wantedBy?: string | string[];
	requiredBy?: string | string[];
	alias?: string | string[];
	also?: string | string[];
	defaultInstance?: string;
}

// Complete systemd service configuration
export interface SystemdServiceContent {
	unit?: UnitConfig;
	service: ServiceConfig;
	install?: InstallConfig;
}

export interface SystemdService extends BaseAction {
	type: "systemdService";
	name: string; // Service name (will add .service if not present)
	content: string | SystemdServiceContent; // Service unit file content (string for backward compatibility)
	user?: boolean; // User service (true) or system service (false)
	enable?: boolean; // Enable the service
	start?: boolean; // Start the service
	reload?: boolean; // Reload systemd daemon after creating
}

export function systemdService(options: Omit<SystemdService, "type">): SystemdService {
	return {
		type: "systemdService",
		...options,
	};
}
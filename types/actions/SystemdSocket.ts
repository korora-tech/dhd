import type { BaseAction } from "./base";
import type { UnitConfig, InstallConfig } from "./SystemdService";

// Socket configuration
export interface SocketConfig {
	// Socket listening configuration
	listenStream?: string | string[]; // TCP socket (address:port or path)
	listenDatagram?: string | string[]; // UDP socket
	listenSequentialPacket?: string | string[]; // Unix sequential packet socket
	listenFIFO?: string | string[]; // FIFO
	listenSpecial?: string | string[]; // Special file
	listenNetlink?: string | string[]; // Netlink socket
	listenMessageQueue?: string | string[]; // POSIX message queue
	listenUSBFunction?: string | string[]; // USB FunctionFS endpoint
	
	// Socket options
	bindIPv6Only?: "default" | "both" | "ipv6-only";
	backlog?: number;
	bindToDevice?: string;
	socketUser?: string;
	socketGroup?: string;
	socketMode?: string; // Octal mode like "0666"
	directoryMode?: string; // Octal mode for parent directories
	accept?: boolean;
	writable?: boolean;
	maxConnections?: number;
	maxConnectionsPerSource?: number;
	keepAlive?: boolean;
	keepAliveTimeSec?: number;
	keepAliveIntervalSec?: number;
	keepAliveProbes?: number;
	noDelay?: boolean;
	priority?: number;
	deferAcceptSec?: number;
	receiveBuffer?: string;
	sendBuffer?: string;
	ipTOS?: number;
	ipTTL?: number;
	mark?: number;
	reusePort?: boolean;
	smackLabel?: string;
	smackLabelIPIn?: string;
	smackLabelIPOut?: string;
	seLinuxContextFromNet?: boolean;
	pipeSize?: string;
	messageQueueMaxMessages?: number;
	messageQueueMessageSize?: number;
	freeBind?: boolean;
	transparent?: boolean;
	broadcast?: boolean;
	passCredentials?: boolean;
	passecurity?: boolean;
	tcpCongestion?: string;
	execStartPre?: string | string[];
	execStartPost?: string | string[];
	execStopPre?: string | string[];
	execStopPost?: string | string[];
	timeoutSec?: number;
	service?: string; // Service to activate
	removeOnStop?: boolean;
	symlinks?: string | string[];
	fileDescriptorName?: string;
	triggerLimitIntervalSec?: number;
	triggerLimitBurst?: number;
}

// Complete systemd socket configuration
export interface SystemdSocketContent {
	unit?: UnitConfig;
	socket: SocketConfig;
	install?: InstallConfig;
}

export interface SystemdSocket extends BaseAction {
	type: "systemdSocket";
	name: string; // Socket name (will add .socket if not present)
	content: string | SystemdSocketContent; // Socket unit file content (string for backward compatibility)
	user?: boolean; // User socket (true) or system socket (false)
	enable?: boolean; // Enable the socket
	start?: boolean; // Start the socket
	reload?: boolean; // Reload systemd daemon after creating
}

export function systemdSocket(options: Omit<SystemdSocket, "type">): SystemdSocket {
	return {
		type: "systemdSocket",
		...options,
	};
}
import type { BaseAction } from "./base";

export interface SystemdSocket extends BaseAction {
	type: "systemdSocket";
	name: string; // Socket name (will add .socket if not present)
	content: string; // Socket unit file content
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
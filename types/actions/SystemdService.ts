import type { BaseAction } from "./base";

export interface SystemdService extends BaseAction {
	type: "systemdService";
	name: string; // Service name (will add .service if not present)
	content: string; // Service unit file content
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
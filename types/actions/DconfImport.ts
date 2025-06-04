import type { BaseAction } from "./base";

export interface DconfImport extends BaseAction {
	type: "dconfImport";
	source: string; // Path to dconf file
	path: string; // Dconf path like "/org/gnome/desktop/interface/"
	backup?: boolean;
}

export function dconfImport(options: Omit<DconfImport, "type">): DconfImport {
	return {
		type: "dconfImport",
		...options,
	};
}
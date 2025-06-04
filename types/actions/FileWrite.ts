import type { BaseAction } from "./base";

export interface FileWrite extends BaseAction {
	type: "fileWrite";
	destination: string;
	content: string;
	mode?: string; // Unix file permissions like "644" or "755"
	privileged?: boolean;
	backup?: boolean;
}

export function fileWrite(options: Omit<FileWrite, "type">): FileWrite {
	return {
		type: "fileWrite",
		...options,
	};
}
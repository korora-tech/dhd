import type { BaseAction } from "./base";

export interface CopyFile extends BaseAction {
	type: "copyFile";
	source: string;
	destination: string;
	privileged?: boolean;
	mode?: string; // Unix file permissions like "644" or "755"
	backup?: boolean;
}

export function copyFile(options: Omit<CopyFile, "type">): CopyFile {
	return {
		type: "copyFile",
		...options,
	};
}
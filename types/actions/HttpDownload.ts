import type { BaseAction } from "./base";

export interface HttpDownload extends BaseAction {
	type: "httpDownload";
	url: string;
	destination: string;
	checksum?: string;
	checksumType?: "sha256" | "sha512" | "md5";
	mode?: string; // Unix file permissions like "644" or "755"
	privileged?: boolean;
}

export function httpDownload(options: Omit<HttpDownload, "type">): HttpDownload {
	return {
		type: "httpDownload",
		...options,
	};
}
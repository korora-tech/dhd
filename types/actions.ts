export * from "./actions/base";
export * from "./actions/PackageInstall";
export * from "./actions/ExecuteCommand";
export * from "./actions/LinkDotfile";
export * from "./actions/CopyFile";
export * from "./actions/HttpDownload";
export * from "./actions/FileWrite";
export * from "./actions/DconfImport";
export * from "./actions/SystemdService";
export * from "./actions/SystemdSocket";

import type { PackageInstall } from "./actions/PackageInstall";
import type { ExecuteCommand } from "./actions/ExecuteCommand";
import type { LinkDotfile } from "./actions/LinkDotfile";
import type { CopyFile } from "./actions/CopyFile";
import type { HttpDownload } from "./actions/HttpDownload";
import type { FileWrite } from "./actions/FileWrite";
import type { DconfImport } from "./actions/DconfImport";
import type { SystemdService } from "./actions/SystemdService";
import type { SystemdSocket } from "./actions/SystemdSocket";

export type AnyAction =
	| PackageInstall
	| ExecuteCommand
	| LinkDotfile
	| CopyFile
	| HttpDownload
	| FileWrite
	| DconfImport
	| SystemdService
	| SystemdSocket;

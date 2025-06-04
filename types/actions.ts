export * from "./actions/base";
export * from "./actions/PackageInstall";
export * from "./actions/ExecuteCommand";
export * from "./actions/LinkDotfile";
export * from "./actions/CopyFile";
export * from "./actions/HttpDownload";
export * from "./actions/FileWrite";

import type { PackageInstall } from "./actions/PackageInstall";
import type { ExecuteCommand } from "./actions/ExecuteCommand";
import type { LinkDotfile } from "./actions/LinkDotfile";
import type { CopyFile } from "./actions/CopyFile";
import type { HttpDownload } from "./actions/HttpDownload";
import type { FileWrite } from "./actions/FileWrite";

export type AnyAction =
	| PackageInstall
	| ExecuteCommand
	| LinkDotfile
	| CopyFile
	| HttpDownload
	| FileWrite;

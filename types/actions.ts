export * from "./actions/base";
export * from "./actions/PackageInstall";
export * from "./actions/ExecuteCommand";
export * from "./actions/LinkDotfile";
export * from "./actions/Dotfile";

import type { PackageInstall } from "./actions/PackageInstall";
import type { ExecuteCommand } from "./actions/ExecuteCommand";
import type { LinkDotfile } from "./actions/LinkDotfile";

export type AnyAction =
	| PackageInstall
	| ExecuteCommand
	| LinkDotfile;

export * from "./actions/base";
export * from "./actions/PackageInstall";
export * from "./actions/ExecuteCommand";
export * from "./actions/LinkDotfile";

import type { PackageInstall } from "./actions/PackageInstall";
import type { ExecuteCommand } from "./actions/ExecuteCommand";
import type { LinkDotfile } from "./actions/LinkDotfile";

export type AnyAction =
	| PackageInstall
	| ExecuteCommand
	| LinkDotfile;

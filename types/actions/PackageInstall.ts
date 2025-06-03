import { Action } from "./base";

export type PackageNames = string | string[];

export type SystemPackageManager = "apt" | "brew" | "pacman" | "dnf" | "yum" | "zypper" | "nix";
export type LanguagePackageManager = "npm" | "yarn" | "pnpm" | "pip" | "pipx" | "gem" | "cargo" | "go" | "composer";
export type PackageManager = SystemPackageManager | LanguagePackageManager;

export interface SystemPackageOptions {
	names: PackageNames;
	manager?: SystemPackageManager;
}

export interface LanguagePackageOptions {
	[K: string]: string | string[];
}

export type PackageInstallOptions = SystemPackageOptions | LanguagePackageOptions;

export interface PackageInstall extends Action {
	type: "PackageInstall";
	options: PackageInstallOptions;
}

export function packageInstall(options: PackageInstallOptions): PackageInstall {
	return {
		type: "PackageInstall",
		options
	};
}
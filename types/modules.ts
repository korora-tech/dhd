import { AnyAction } from "./actions";

export type LinuxFamily = "debian" | "redhat" | "suse";
export type LinuxDistro = "ubuntu" | "debian" | "fedora" | "rhel" | "centos" | "arch" | "manjaro" | "opensuse" | "nixos";
export type Platform = "linux" | "mac" | "windows";

export type PackageValue = string | string[];

export interface LinuxOptions {
	family?: Partial<Record<LinuxFamily, PackageValue>>;
	distro?: Partial<Record<LinuxDistro, PackageValue>>;
}

export interface PlatformOptions {
	default: PackageValue;
	windows?: PackageValue;
	mac?: PackageValue;
	linux?: PackageValue | LinuxOptions;
}

export interface PlatformSelector {
	select(options: PlatformOptions): string[];
}

export interface SystemContext {
	os: string;
	arch: string;
	home: string;
	platform: PlatformSelector;
	
	fileExists(path: string): Promise<boolean>;
	readFile(path: string): Promise<string>;
	commandExists(command: string): Promise<boolean>;
	getEnv(key: string): string | undefined;
}

export interface Module {
	type: "module";
	name: string;
	description?: string;
	dependencies: string[];
	setup: (context: SystemContext) => AnyAction[];
}

export class ModuleBuilder {
	private config: {
		name: string;
		description?: string;
		dependencies: string[];
		setup?: (context: SystemContext) => AnyAction[];
	};

	constructor(name: string) {
		this.config = {
			name,
			dependencies: []
		};
	}

	description(desc: string): this {
		this.config.description = desc;
		return this;
	}

	depends(...deps: string[]): this {
		this.config.dependencies.push(...deps);
		return this;
	}

	with(setup: (context: SystemContext) => AnyAction[]): Module {
		this.config.setup = setup;
		return {
			type: "module",
			name: this.config.name,
			description: this.config.description,
			dependencies: this.config.dependencies,
			setup: this.config.setup || (() => [])
		};
	}
}

export function defineModule(name: string): ModuleBuilder {
	return new ModuleBuilder(name);
}
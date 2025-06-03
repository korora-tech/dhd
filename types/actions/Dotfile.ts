import { Action } from "./base";

export interface DotfileOptions {
	source: string;
	target: string;
	backup?: boolean;
}

export interface Dotfile extends Action {
	type: "Dotfile";
	options: DotfileOptions;
}

export function dotfile(options: DotfileOptions): Dotfile {
	return {
		type: "Dotfile",
		options
	};
}
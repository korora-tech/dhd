import { Action } from "./base";

export interface LinkDotfileOptions {
  source: string;
  target: string;
  backup?: boolean;
  force?: boolean;
}

export interface LinkDotfile extends Action {
  type: "LinkDotfile";
  options: LinkDotfileOptions;
}

export function linkDotfile(options: LinkDotfileOptions): LinkDotfile {
  return {
    type: "LinkDotfile",
    options
  };
}
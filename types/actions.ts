export * from "./actions/base";
export * from "./actions/PackageInstall";
export * from "./actions/ExecuteCommand";
export * from "./actions/LinkDotfile";
export * from "./actions/Dotfile";

import { PackageInstall } from "./actions/PackageInstall";
import { ExecuteCommand } from "./actions/ExecuteCommand";
import { LinkDotfile } from "./actions/LinkDotfile";
import { Dotfile } from "./actions/Dotfile";

export type AnyAction = 
  | PackageInstall
  | ExecuteCommand
  | LinkDotfile
  | Dotfile;
import { Action } from "./base";

export interface ExecuteCommandOptions {
  command: string;
  args?: string[];
  cwd?: string;
  env?: Record<string, string>;
  failOnError?: boolean;
}

export interface ExecuteCommand extends Action {
  type: "ExecuteCommand";
  options: ExecuteCommandOptions;
}

export function executeCommand(options: ExecuteCommandOptions): ExecuteCommand {
  return {
    type: "ExecuteCommand",
    options
  };
}
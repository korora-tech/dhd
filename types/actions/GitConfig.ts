import { BaseAction } from "./base";

export interface GitConfig extends BaseAction {
  type: "gitConfig";
  scope: "global" | "system" | "local";
  configs: Record<string, string>;
}

export function gitConfig(options: Omit<GitConfig, "type">): GitConfig {
  return { type: "gitConfig", ...options };
}
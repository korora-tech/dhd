import type { BaseAction } from "./base";

export interface UserGroup extends BaseAction {
	type: "userGroup";
	user: string; // Username to modify
	groups: string[]; // Groups to add the user to
	append?: boolean; // Append to existing groups (default: true)
}

export function userGroup(options: Omit<UserGroup, "type">): UserGroup {
	return {
		type: "userGroup",
		append: true,
		...options,
	};
}
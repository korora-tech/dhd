import { defineModule, copyFile } from "@korora-tech/dhd";

export default defineModule("copyFile")
	.description("Example of copyFile")
	.with(() => [
		// Basic file copy
		copyFile({
			source: "config.toml",
			destination: "~/.config/myapp/config.toml",
		}),

		// Copy system file with elevated privileges
		copyFile({
			source: "hosts",
			destination: "/etc/hosts",
			privileged: true,
			backup: true,
		}),

		// Copy with specific permissions
		copyFile({
			source: "script.sh",
			destination: "~/bin/script.sh",
			mode: "755",
		}),

		// Copy sensitive file with restricted permissions
		copyFile({
			source: "secrets.conf",
			destination: "/etc/myapp/secrets.conf",
			privileged: true,
			mode: "600",
			backup: true,
		}),
	]);

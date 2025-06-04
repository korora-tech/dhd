import { defineModule, dconfImport } from "@korora-tech/dhd";

export default defineModule("dconfImport")
	.description("Example of dconfImport")
	.with(() => [
		// Import GNOME desktop settings
		dconfImport({
			source: "gnome-desktop.conf",
			path: "/org/gnome/desktop/interface/",
			backup: true,
		}),

		// Import GNOME shell settings
		dconfImport({
			source: "gnome-shell.conf",
			path: "/org/gnome/shell/",
		}),

		// Import GNOME keybindings
		dconfImport({
			source: "keybindings.conf",
			path: "/org/gnome/desktop/wm/keybindings/",
			backup: true,
		}),

		// Import terminal preferences
		dconfImport({
			source: "gnome-terminal.conf",
			path: "/org/gnome/terminal/",
		}),

		// Import custom application settings
		dconfImport({
			source: "custom-apps.conf",
			path: "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/",
			backup: true,
		}),
	]);
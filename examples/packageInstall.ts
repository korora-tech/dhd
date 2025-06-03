import { defineModule, packageInstall } from "../types";

export default defineModule("packageInstall")
	.description("Example of using packageInstall")
	.with((ctx) => [
		// Basic package installation
		packageInstall({
			names: ["neovim"]
		}),


		// Package installation with support for multiple platforms
		packageInstall({
			names: ctx.platform.select({
				default: "neovim",
				mac: "neovim",
				windows: "Neovim.Neovim",
			})
		}),

		// Example with Linux-specific distribution handling
		packageInstall({
			names: ctx.platform.select({
				default: "htop", // default for any platform or distro
				linux: {
					family: {
						redhat: "htop",
						debian: "btop"
						// suse not specified - will use default "htop"
					},
					distro: {
						arch: "htop-git", // specific package for Arch Linux
					}
				}
			})
		}),

		// Language-specific package managers
		packageInstall({
			go: "github.com/runmedev/runme@latest",
		}),
		packageInstall({
			cargo: "zellij",
		}),
	]);

// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import cloudflare from '@astrojs/cloudflare';

// https://astro.build/config
export default defineConfig({
	output: "static",
	adapter: cloudflare(),
	integrations: [
		starlight({
			title: "DHD",
			description:
				"Declarative Home Deployment - Manage your home directory, dotfiles, and system configuration using TypeScript",
			social: [
				{
					icon: "github",
					label: "GitHub",
					href: "https://github.com/korora-tech/dhd",
				},
			],
			sidebar: [
				{
					label: "Getting Started",
					items: [
						{ label: "Introduction", slug: "getting-started/introduction" },
						{ label: "Installation", slug: "getting-started/installation" },
						{ label: "Quick Start", slug: "getting-started/quick-start" },
					],
				},
				{
					label: "Guides",
					items: [{ label: "Example Guide", slug: "guides/example" }],
				},
				{
					label: "Reference",
					items: [{ label: "Example Reference", slug: "reference/example" }],
				},
			],
		}),
	],
});
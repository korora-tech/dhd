// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import cloudflare from '@astrojs/cloudflare';

// https://astro.build/config
export default defineConfig({
	output: 'server',
	adapter: cloudflare({
		mode: 'directory',
		functionPerRoute: false
	}),
	integrations: [
		starlight({
			title: 'DHD',
			description: 'Declarative Home Deployment - Manage your home directory, dotfiles, and system configuration using TypeScript',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/korora-tech/dhd' }
			],
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Introduction', slug: 'getting-started/introduction' },
						{ label: 'Installation', slug: 'getting-started/installation' },
						{ label: 'Quick Start', slug: 'getting-started/quick-start' },
					],
				},
				{
					label: 'Core Concepts',
					items: [
						{ label: 'Modules', slug: 'concepts/modules' },
						{ label: 'Actions', slug: 'concepts/actions' },
						{ label: 'Platform Detection', slug: 'concepts/platform-detection' },
						{ label: 'Execution Model', slug: 'concepts/execution-model' },
					],
				},
				{
					label: 'Action Reference',
					autogenerate: { directory: 'reference/actions' },
				},
				{
					label: 'Examples',
					items: [
						{ label: 'Dotfiles Management', slug: 'examples/dotfiles' },
						{ label: 'Development Environment', slug: 'examples/dev-environment' },
						{ label: 'System Services', slug: 'examples/system-services' },
						{ label: 'Desktop Configuration', slug: 'examples/desktop-config' },
					],
				},
				{
					label: 'CLI Reference',
					items: [
						{ label: 'Commands', slug: 'cli/commands' },
						{ label: 'Configuration', slug: 'cli/configuration' },
					],
				},
				{
					label: 'Advanced',
					items: [
						{ label: 'Architecture', slug: 'advanced/architecture' },
						{ label: 'Writing Custom Actions', slug: 'advanced/custom-actions' },
						{ label: 'Contributing', slug: 'advanced/contributing' },
					],
				},
			],
		}),
	],
});

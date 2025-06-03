import { defineConfig } from 'vite';

export default defineConfig({
	root: 'web',
	build: {
		outDir: '../dist',
		emptyOutDir: true,
		target: 'esnext'
	},
	server: {
		port: 3000
	}
});
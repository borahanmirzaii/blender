import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		fs: {
			// Allow serving WASM files from the workspace
			allow: ['../..']
		}
	},
	optimizeDeps: {
		exclude: ['blender-wasm']
	}
});

<script lang="ts">
	import { onMount } from 'svelte';
	import Viewport3D from '$lib/components/Viewport3D.svelte';
	import Outliner from '$lib/components/Outliner.svelte';
	import Properties from '$lib/components/Properties.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import { sceneStore } from '$lib/stores/scene.svelte';

	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			// Dynamically import the WASM module
			const wasmModule = await import('/wasm/blender_wasm.js');
			await wasmModule.default('/wasm/blender_wasm_bg.wasm');

			// Create default scene (cube + light + camera)
			const scene = wasmModule.SceneHandle.defaultScene();
			sceneStore.init(scene);
			loading = false;
		} catch (e: any) {
			console.error('WASM init failed:', e);
			error = e.message || 'Failed to load WASM module';
			loading = false;
		}
	});

	async function onFileDrop(e: DragEvent) {
		e.preventDefault();
		const file = e.dataTransfer?.files[0];
		if (!file) return;

		if (file.name.endsWith('.glb') || file.name.endsWith('.gltf')) {
			try {
				const wasmModule = await import('/wasm/blender_wasm.js');
				const bytes = new Uint8Array(await file.arrayBuffer());
				const scene = wasmModule.SceneHandle.importGltf(bytes);
				sceneStore.init(scene);
			} catch (e: any) {
				console.error('Import failed:', e);
			}
		}
	}

	function onDragOver(e: DragEvent) { e.preventDefault(); }
</script>

<div class="app" ondrop={onFileDrop} ondragover={onDragOver} role="application">
	{#if loading}
		<div class="loading">
			<div class="spinner"></div>
			<p>Loading Blender Web (586KB WASM)...</p>
		</div>
	{:else if error}
		<div class="error">
			<h2>Failed to initialize</h2>
			<p>{error}</p>
			<p>Make sure your browser supports WebAssembly.</p>
		</div>
	{:else}
		<!-- Header -->
		<header class="header">
			<span class="logo">&#x2B22; Blender Web</span>
			<nav>
				<button>File</button>
				<button>Edit</button>
				<button>Add</button>
			</nav>
			<span class="status">
				{sceneStore.objects.length} objects |
				{sceneStore.handle?.meshCount() ?? 0} meshes |
				WASM 586KB
			</span>
		</header>

		<!-- Main workspace -->
		<div class="workspace">
			<Toolbar />

			<div class="viewport-area">
				<Viewport3D />
				<div class="drop-hint">Drop .glb/.gltf to import</div>
			</div>

			<div class="sidebar">
				<div class="panel">
					<div class="panel-header">Outliner</div>
					<Outliner />
				</div>
				<div class="panel">
					<div class="panel-header">Properties</div>
					<Properties />
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.app { display: flex; flex-direction: column; height: 100vh; overflow: hidden; }
	.loading, .error {
		flex: 1; display: flex; flex-direction: column;
		align-items: center; justify-content: center; gap: 12px;
		color: var(--text-secondary);
	}
	.error h2 { color: var(--danger); }
	.spinner {
		width: 32px; height: 32px; border: 3px solid var(--border);
		border-top-color: var(--accent); border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }

	.header {
		display: flex; align-items: center; height: 32px;
		background: var(--bg-header); border-bottom: 1px solid var(--border);
		padding: 0 10px; gap: 16px; flex-shrink: 0;
	}
	.logo { font-weight: 700; color: var(--accent); font-size: 13px; }
	.header nav { display: flex; gap: 2px; }
	.header nav button {
		border: none; background: transparent; padding: 4px 10px; font-size: 12px;
	}
	.header nav button:hover { background: var(--bg-tertiary); }
	.status { margin-left: auto; font-size: 11px; color: var(--text-muted); font-family: var(--font-mono); }

	.workspace { display: flex; flex: 1; overflow: hidden; }
	.viewport-area { flex: 1; position: relative; overflow: hidden; }
	.drop-hint {
		position: absolute; bottom: 6px; right: 8px;
		font-size: 10px; color: var(--text-muted); pointer-events: none; opacity: 0.5;
	}

	.sidebar {
		width: 260px; display: flex; flex-direction: column;
		border-left: 1px solid var(--border); flex-shrink: 0; overflow-y: auto;
	}
	.panel { border-bottom: 1px solid var(--border); }
	.panel-header {
		padding: 6px 10px; background: var(--bg-header);
		font-size: 11px; font-weight: 600; text-transform: uppercase;
		letter-spacing: 0.5px; color: var(--text-secondary);
	}
</style>

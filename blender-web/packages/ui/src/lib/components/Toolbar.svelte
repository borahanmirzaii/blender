<script lang="ts">
	import { sceneStore } from '$lib/stores/scene.svelte';

	let activeTool = $state('select');

	const tools = [
		{ id: 'select', label: 'Select', icon: '\u2732' },
		{ id: 'move', label: 'Move', icon: '\u2725' },
		{ id: 'rotate', label: 'Rotate', icon: '\u21BB' },
		{ id: 'scale', label: 'Scale', icon: '\u2922' },
	];

	const addItems = [
		{ type: 'cube', label: 'Cube' },
		{ type: 'sphere', label: 'Sphere' },
	];

	function addPrimitive(type: string) {
		const name = type.charAt(0).toUpperCase() + type.slice(1);
		sceneStore.addPrimitive(type, name);
	}

	function deleteSelected() {
		sceneStore.deleteActive();
	}

	function exportScene() {
		const json = sceneStore.exportJson();
		if (json) {
			const blob = new Blob([json], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = 'scene.json';
			a.click();
			URL.revokeObjectURL(url);
		}
	}
</script>

<div class="toolbar">
	<!-- Transform tools -->
	<div class="tool-group">
		{#each tools as tool}
			<button
				class="tool-btn"
				class:active={activeTool === tool.id}
				onclick={() => activeTool = tool.id}
				title={tool.label}
			>
				{tool.icon}
			</button>
		{/each}
	</div>

	<div class="separator"></div>

	<!-- Add primitives -->
	<div class="tool-group">
		{#each addItems as item}
			<button
				class="tool-btn"
				onclick={() => addPrimitive(item.type)}
				title={`Add ${item.label}`}
			>
				+{item.label.charAt(0)}
			</button>
		{/each}
	</div>

	<div class="separator"></div>

	<!-- Actions -->
	<button class="tool-btn danger" onclick={deleteSelected} title="Delete">
		\u2715
	</button>

	<div class="spacer"></div>

	<button class="tool-btn" onclick={exportScene} title="Export JSON">
		\u21E9
	</button>
</div>

<style>
	.toolbar {
		display: flex;
		flex-direction: column;
		width: 36px;
		background: var(--bg-secondary);
		border-right: 1px solid var(--border);
		padding: 4px;
		gap: 2px;
		align-items: center;
		flex-shrink: 0;
	}

	.tool-group {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.tool-btn {
		width: 28px;
		height: 28px;
		padding: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 14px;
		border-radius: 4px;
	}

	.tool-btn.active {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
	}

	.tool-btn.danger:hover {
		background: var(--danger);
		border-color: var(--danger);
		color: white;
	}

	.separator {
		width: 20px;
		height: 1px;
		background: var(--border);
		margin: 4px 0;
	}

	.spacer { flex: 1; }
</style>

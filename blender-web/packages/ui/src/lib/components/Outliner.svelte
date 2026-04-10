<script lang="ts">
	import { sceneStore } from '$lib/stores/scene.svelte';

	function selectObject(index: number) {
		sceneStore.selectObject(index);
	}

	function getIcon(type: string): string {
		switch (type) {
			case 'mesh': return '\u25A6';    // dotted square
			case 'light': return '\u2600';   // sun
			case 'camera': return '\u25A3';  // square with fill
			default: return '\u25CB';         // circle
		}
	}
</script>

<div class="outliner">
	<div class="scene-root">
		<span class="tree-icon">\u25BC</span>
		<span class="scene-name">Scene Collection</span>
	</div>

	{#each sceneStore.objects as obj, i}
		<button
			class="tree-item"
			class:selected={obj.selected}
			onclick={() => selectObject(i)}
		>
			<span class="indent"></span>
			<span class="obj-icon">{getIcon(obj.type)}</span>
			<span class="obj-name">{obj.name}</span>
			<button
				class="visibility-toggle"
				class:hidden={!obj.visible}
				onclick|stopPropagation={() => {}}
			>
				{obj.visible ? '\u25C9' : '\u25CE'}
			</button>
		</button>
	{/each}

	{#if sceneStore.objects.length === 0}
		<div class="empty-state">No objects in scene</div>
	{/if}
</div>

<style>
	.outliner {
		padding: 4px 0;
	}

	.scene-root {
		display: flex;
		align-items: center;
		padding: 4px 8px;
		gap: 4px;
		font-size: 12px;
		color: var(--text-secondary);
	}

	.tree-icon { font-size: 8px; }

	.tree-item {
		display: flex;
		align-items: center;
		width: 100%;
		padding: 3px 8px;
		gap: 4px;
		border: none;
		background: transparent;
		text-align: left;
		font-size: 12px;
		color: var(--text-primary);
		cursor: pointer;
	}

	.tree-item:hover {
		background: var(--bg-tertiary);
	}

	.tree-item.selected {
		background: var(--selection);
		color: var(--accent);
	}

	.indent { width: 16px; flex-shrink: 0; }
	.obj-icon { width: 16px; text-align: center; flex-shrink: 0; }
	.obj-name { flex: 1; overflow: hidden; text-overflow: ellipsis; }

	.visibility-toggle {
		border: none;
		background: none;
		color: var(--text-muted);
		font-size: 12px;
		padding: 0 4px;
		cursor: pointer;
	}

	.visibility-toggle:hover { color: var(--text-primary); }
	.visibility-toggle.hidden { opacity: 0.3; }

	.empty-state {
		padding: 16px;
		text-align: center;
		color: var(--text-muted);
		font-size: 11px;
	}
</style>

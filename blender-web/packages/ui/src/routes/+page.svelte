<script lang="ts">
	import Viewport3D from '$lib/components/Viewport3D.svelte';
	import Outliner from '$lib/components/Outliner.svelte';
	import Properties from '$lib/components/Properties.svelte';
	import Timeline from '$lib/components/Timeline.svelte';
	import AIAssistant from '$lib/components/AIAssistant.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import { sceneStore } from '$lib/stores/scene.svelte';

	let showAI = $state(false);
</script>

<div class="blender-layout">
	<!-- Top header bar -->
	<header class="header-bar">
		<div class="logo">
			<span class="logo-icon">&#x2B22;</span>
			<span>Blender Web</span>
		</div>
		<nav class="menu-bar">
			<button>File</button>
			<button>Edit</button>
			<button>View</button>
			<button>Add</button>
			<button>Render</button>
		</nav>
		<div class="header-right">
			<button class="ai-toggle" onclick={() => showAI = !showAI}>
				{showAI ? 'Close AI' : 'AI Assistant'}
			</button>
			<span class="status">WASM Ready | WebGPU</span>
		</div>
	</header>

	<!-- Main workspace -->
	<div class="workspace">
		<!-- Left toolbar -->
		<Toolbar />

		<!-- Center: 3D Viewport -->
		<div class="viewport-area">
			<Viewport3D />
		</div>

		<!-- Right panel: Outliner + Properties -->
		<div class="right-panel">
			<div class="panel-section outliner">
				<div class="panel-header">Outliner</div>
				<Outliner />
			</div>
			<div class="panel-section properties">
				<div class="panel-header">Properties</div>
				<Properties />
			</div>
		</div>

		<!-- AI Assistant panel (toggleable) -->
		{#if showAI}
			<div class="ai-panel">
				<AIAssistant />
			</div>
		{/if}
	</div>

	<!-- Bottom: Timeline -->
	<div class="timeline-area">
		<Timeline />
	</div>
</div>

<style>
	.blender-layout {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
	}

	.header-bar {
		display: flex;
		align-items: center;
		height: 32px;
		background: var(--bg-header);
		border-bottom: 1px solid var(--border);
		padding: 0 8px;
		gap: 16px;
		flex-shrink: 0;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 6px;
		font-weight: 600;
		color: var(--accent);
	}

	.logo-icon { font-size: 16px; }

	.menu-bar {
		display: flex;
		gap: 2px;
	}

	.menu-bar button {
		border: none;
		background: transparent;
		padding: 4px 10px;
		font-size: 12px;
	}

	.menu-bar button:hover {
		background: var(--bg-tertiary);
	}

	.header-right {
		margin-left: auto;
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.ai-toggle {
		background: var(--accent);
		border-color: var(--accent);
		color: white;
		font-size: 11px;
		padding: 2px 10px;
	}

	.status {
		font-size: 11px;
		color: var(--text-muted);
	}

	.workspace {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.viewport-area {
		flex: 1;
		position: relative;
		overflow: hidden;
	}

	.right-panel {
		width: 280px;
		display: flex;
		flex-direction: column;
		border-left: 1px solid var(--border);
		overflow-y: auto;
		flex-shrink: 0;
	}

	.panel-section {
		border-bottom: 1px solid var(--border);
	}

	.panel-header {
		padding: 6px 10px;
		background: var(--bg-header);
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--text-secondary);
	}

	.outliner { flex: 0 0 200px; overflow-y: auto; }
	.properties { flex: 1; overflow-y: auto; }

	.ai-panel {
		width: 360px;
		border-left: 1px solid var(--border);
		flex-shrink: 0;
		overflow: hidden;
	}

	.timeline-area {
		height: 120px;
		border-top: 1px solid var(--border);
		flex-shrink: 0;
	}
</style>

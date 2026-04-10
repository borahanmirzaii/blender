<script lang="ts">
	import { sceneStore } from '$lib/stores/scene.svelte';

	let posX = $state(0);
	let posY = $state(0);
	let posZ = $state(0);

	$effect(() => {
		const obj = sceneStore.activeObject;
		if (obj) {
			posX = obj.position[0];
			posY = obj.position[1];
			posZ = obj.position[2];
		}
	});

	function updatePosition() {
		if (sceneStore.activeObjectIndex !== null) {
			sceneStore.setPosition(sceneStore.activeObjectIndex, posX, posY, posZ);
		}
	}
</script>

<div class="properties-panel">
	{#if sceneStore.activeObject}
		{@const obj = sceneStore.activeObject}

		<!-- Object header -->
		<div class="prop-section">
			<div class="section-header">Object</div>
			<div class="prop-row">
				<label>Name</label>
				<input type="text" value={obj.name} readonly />
			</div>
			<div class="prop-row">
				<label>Type</label>
				<span class="prop-value">{obj.type}</span>
			</div>
		</div>

		<!-- Transform -->
		<div class="prop-section">
			<div class="section-header">Transform</div>
			<div class="prop-row">
				<label>Location</label>
				<div class="vec3-input">
					<div class="vec-field">
						<span class="axis x">X</span>
						<input type="number" bind:value={posX} onchange={updatePosition} step="0.1" />
					</div>
					<div class="vec-field">
						<span class="axis y">Y</span>
						<input type="number" bind:value={posY} onchange={updatePosition} step="0.1" />
					</div>
					<div class="vec-field">
						<span class="axis z">Z</span>
						<input type="number" bind:value={posZ} onchange={updatePosition} step="0.1" />
					</div>
				</div>
			</div>
		</div>

		<!-- Material (for mesh objects) -->
		{#if obj.type === 'mesh'}
			<div class="prop-section">
				<div class="section-header">Material</div>
				<div class="prop-row">
					<label>Base Color</label>
					<input type="color" value="#cccccc" />
				</div>
				<div class="prop-row">
					<label>Metallic</label>
					<input type="range" min="0" max="1" step="0.01" value="0" />
				</div>
				<div class="prop-row">
					<label>Roughness</label>
					<input type="range" min="0" max="1" step="0.01" value="0.5" />
				</div>
			</div>
		{/if}
	{:else}
		<div class="empty-state">
			Select an object to view properties
		</div>
	{/if}

	<!-- Scene properties -->
	<div class="prop-section">
		<div class="section-header">Scene</div>
		<div class="prop-row">
			<label>Objects</label>
			<span class="prop-value">{sceneStore.objects.length}</span>
		</div>
		<div class="prop-row">
			<label>FPS</label>
			<span class="prop-value">{sceneStore.fps}</span>
		</div>
		<div class="prop-row">
			<label>Frames</label>
			<span class="prop-value">{sceneStore.frameStart} - {sceneStore.frameEnd}</span>
		</div>
	</div>
</div>

<style>
	.properties-panel {
		padding: 4px;
	}

	.prop-section {
		margin-bottom: 2px;
	}

	.section-header {
		padding: 4px 8px;
		font-size: 11px;
		font-weight: 600;
		color: var(--text-secondary);
		background: var(--bg-primary);
		border-bottom: 1px solid var(--border);
	}

	.prop-row {
		display: flex;
		align-items: flex-start;
		padding: 4px 8px;
		gap: 8px;
	}

	.prop-row label {
		width: 70px;
		font-size: 11px;
		color: var(--text-secondary);
		flex-shrink: 0;
		padding-top: 3px;
	}

	.prop-row input[type="text"],
	.prop-row input[type="number"] {
		flex: 1;
		width: 100%;
		font-size: 11px;
		padding: 2px 6px;
	}

	.prop-row input[type="range"] {
		flex: 1;
	}

	.prop-row input[type="color"] {
		width: 60px;
		height: 22px;
		padding: 0;
		border: 1px solid var(--border);
	}

	.prop-value {
		font-size: 11px;
		color: var(--text-primary);
		font-family: var(--font-mono);
	}

	.vec3-input {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
	}

	.vec-field {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.vec-field input {
		flex: 1;
		font-size: 11px;
		padding: 2px 4px;
	}

	.axis {
		font-size: 10px;
		font-weight: bold;
		width: 12px;
		text-align: center;
	}

	.axis.x { color: #e44; }
	.axis.y { color: #4c8; }
	.axis.z { color: #48f; }

	.empty-state {
		padding: 24px 16px;
		text-align: center;
		color: var(--text-muted);
		font-size: 11px;
	}
</style>

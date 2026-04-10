<script lang="ts">
	import { sceneStore } from '$lib/stores/scene.svelte';

	let isPlaying = $state(false);
	let intervalId: ReturnType<typeof setInterval> | null = null;

	function togglePlay() {
		isPlaying = !isPlaying;
		if (isPlaying) {
			intervalId = setInterval(() => {
				let next = sceneStore.frameCurrent + 1;
				if (next > sceneStore.frameEnd) next = sceneStore.frameStart;
				sceneStore.setFrame(next);
			}, 1000 / sceneStore.fps);
		} else if (intervalId) {
			clearInterval(intervalId);
			intervalId = null;
		}
	}

	function goToStart() {
		sceneStore.setFrame(sceneStore.frameStart);
	}

	function goToEnd() {
		sceneStore.setFrame(sceneStore.frameEnd);
	}

	function stepBack() {
		sceneStore.setFrame(sceneStore.frameCurrent - 1);
	}

	function stepForward() {
		sceneStore.setFrame(sceneStore.frameCurrent + 1);
	}

	function onScrub(e: Event) {
		const input = e.target as HTMLInputElement;
		sceneStore.setFrame(parseInt(input.value, 10));
	}
</script>

<div class="timeline">
	<div class="transport-controls">
		<button onclick={goToStart} title="Go to start">|&lt;</button>
		<button onclick={stepBack} title="Step back">&lt;</button>
		<button onclick={togglePlay} title="Play/Pause" class:playing={isPlaying}>
			{isPlaying ? '\u23F8' : '\u25B6'}
		</button>
		<button onclick={stepForward} title="Step forward">&gt;</button>
		<button onclick={goToEnd} title="Go to end">&gt;|</button>

		<div class="frame-display">
			<input
				type="number"
				value={sceneStore.frameCurrent}
				onchange={onScrub}
				min={sceneStore.frameStart}
				max={sceneStore.frameEnd}
			/>
			<span class="frame-label">/ {sceneStore.frameEnd}</span>
		</div>
	</div>

	<div class="timeline-track">
		<input
			type="range"
			min={sceneStore.frameStart}
			max={sceneStore.frameEnd}
			value={sceneStore.frameCurrent}
			oninput={onScrub}
			class="scrubber"
		/>

		<!-- Frame markers -->
		<div class="frame-markers">
			{#each Array(Math.min(25, sceneStore.frameEnd - sceneStore.frameStart + 1)) as _, i}
				{@const frame = sceneStore.frameStart + Math.floor(i * (sceneStore.frameEnd - sceneStore.frameStart) / 24)}
				<span class="marker" class:current={frame === sceneStore.frameCurrent}>
					{frame}
				</span>
			{/each}
		</div>
	</div>
</div>

<style>
	.timeline {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--bg-secondary);
		padding: 4px 8px;
	}

	.transport-controls {
		display: flex;
		align-items: center;
		gap: 2px;
		padding-bottom: 4px;
	}

	.transport-controls button {
		width: 28px;
		height: 24px;
		padding: 0;
		font-size: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.transport-controls button.playing {
		background: var(--accent);
		border-color: var(--accent);
	}

	.frame-display {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-left: 12px;
	}

	.frame-display input {
		width: 50px;
		text-align: center;
		font-family: var(--font-mono);
		font-size: 12px;
	}

	.frame-label {
		font-size: 11px;
		color: var(--text-muted);
	}

	.timeline-track {
		flex: 1;
		display: flex;
		flex-direction: column;
	}

	.scrubber {
		width: 100%;
		accent-color: var(--accent);
	}

	.frame-markers {
		display: flex;
		justify-content: space-between;
		padding: 0 2px;
	}

	.marker {
		font-size: 9px;
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	.marker.current {
		color: var(--accent);
		font-weight: bold;
	}
</style>

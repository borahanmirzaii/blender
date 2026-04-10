<script lang="ts">
	import { onMount } from 'svelte';
	import { sceneStore } from '$lib/stores/scene.svelte';

	let canvas: HTMLCanvasElement;
	let info = $state('Initializing...');

	let isDragging = $state(false);
	let isPanning = $state(false);
	let lastX = 0;
	let lastY = 0;

	onMount(() => {
		startRenderer();
	});

	async function startRenderer() {
		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		info = sceneStore.wasmReady
			? `${sceneStore.objects.length} objects | Canvas2D`
			: 'WASM loading...';

		function frame() {
			if (!canvas || !ctx) return;

			const dpr = window.devicePixelRatio || 1;
			const w = Math.floor(canvas.clientWidth * dpr);
			const h = Math.floor(canvas.clientHeight * dpr);
			if (canvas.width !== w || canvas.height !== h) {
				canvas.width = w;
				canvas.height = h;
			}

			// Dark background
			ctx.fillStyle = '#2a2a2a';
			ctx.fillRect(0, 0, w, h);

			// Grid
			const gridSize = 40 * dpr;
			ctx.strokeStyle = '#333';
			ctx.lineWidth = 1;
			const cx = w / 2, cy = h / 2;
			for (let x = cx % gridSize; x < w; x += gridSize) {
				ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke();
			}
			for (let y = cy % gridSize; y < h; y += gridSize) {
				ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
			}

			// Axes
			ctx.lineWidth = 2 * dpr;
			ctx.strokeStyle = '#c44'; ctx.beginPath(); ctx.moveTo(cx, cy); ctx.lineTo(cx + 60 * dpr, cy); ctx.stroke();
			ctx.strokeStyle = '#4c8'; ctx.beginPath(); ctx.moveTo(cx, cy); ctx.lineTo(cx, cy - 60 * dpr); ctx.stroke();

			// Draw scene objects from WASM
			const handle = sceneStore.handle;
			if (handle) {
				const count = handle.objectCount();
				for (let i = 0; i < count; i++) {
					const mat = handle.objectMatrix(i);
					const tx = mat[12]; // translation X from column-major mat4
					const ty = mat[13]; // translation Y
					const name = handle.objectName(i);
					const type = handle.objectType(i);
					const isActive = i === sceneStore.activeIndex;

					const sx = cx + tx * gridSize;
					const sy = cy - ty * gridSize;

					if (type === 'mesh') {
						// Draw cube/mesh as filled rect
						const size = 20 * dpr;
						ctx.fillStyle = isActive ? '#4b8bbf' : '#777';
						ctx.strokeStyle = isActive ? '#6ab0e8' : '#555';
						ctx.lineWidth = isActive ? 2 * dpr : 1;
						ctx.fillRect(sx - size/2, sy - size/2, size, size);
						ctx.strokeRect(sx - size/2, sy - size/2, size, size);
					} else if (type === 'light') {
						ctx.fillStyle = '#fd3';
						ctx.beginPath(); ctx.arc(sx, sy, 6 * dpr, 0, Math.PI * 2); ctx.fill();
					} else if (type === 'camera') {
						ctx.fillStyle = '#8af';
						ctx.beginPath();
						ctx.moveTo(sx, sy - 8 * dpr);
						ctx.lineTo(sx + 8 * dpr, sy + 6 * dpr);
						ctx.lineTo(sx - 8 * dpr, sy + 6 * dpr);
						ctx.closePath(); ctx.fill();
					}

					// Label
					ctx.fillStyle = isActive ? '#fff' : '#999';
					ctx.font = `${11 * dpr}px sans-serif`;
					ctx.fillText(name, sx + 14 * dpr, sy + 4 * dpr);
				}

				info = `${count} objects | Canvas2D (WebGPU in Phase 2)`;
			}

			requestAnimationFrame(frame);
		}

		requestAnimationFrame(frame);
	}

	function onMouseDown(e: MouseEvent) {
		if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
			isPanning = true;
		} else if (e.button === 0) {
			isDragging = true;
		}
		lastX = e.clientX;
		lastY = e.clientY;
	}

	function onMouseMove(e: MouseEvent) {
		const dx = e.clientX - lastX;
		const dy = e.clientY - lastY;
		lastX = e.clientX;
		lastY = e.clientY;

		const handle = sceneStore.handle;
		if (!handle) return;

		if (isDragging) {
			handle.cameraOrbit(dx, dy);
		} else if (isPanning) {
			handle.cameraPan(dx, dy);
		}
	}

	function onMouseUp() { isDragging = false; isPanning = false; }

	function onWheel(e: WheelEvent) {
		e.preventDefault();
		sceneStore.handle?.cameraZoom(-e.deltaY);
	}

	function onClick(e: MouseEvent) {
		// Simple object picking: click near an object to select it
		const handle = sceneStore.handle;
		if (!handle || isDragging) return;

		const rect = canvas.getBoundingClientRect();
		const dpr = window.devicePixelRatio || 1;
		const mx = (e.clientX - rect.left) * dpr;
		const my = (e.clientY - rect.top) * dpr;
		const cx = canvas.width / 2;
		const cy = canvas.height / 2;
		const gridSize = 40 * dpr;

		let closest = -1;
		let closestDist = 30 * dpr; // max click distance

		const count = handle.objectCount();
		for (let i = 0; i < count; i++) {
			const mat = handle.objectMatrix(i);
			const sx = cx + mat[12] * gridSize;
			const sy = cy - mat[13] * gridSize;
			const dist = Math.hypot(mx - sx, my - sy);
			if (dist < closestDist) {
				closestDist = dist;
				closest = i;
			}
		}

		sceneStore.selectObject(closest >= 0 ? closest : null);
	}
</script>

<div class="viewport">
	<canvas
		bind:this={canvas}
		onmousedown={onMouseDown}
		onmousemove={onMouseMove}
		onmouseup={onMouseUp}
		onmouseleave={onMouseUp}
		onclick={onClick}
		onwheel={onWheel}
	></canvas>
	<div class="info">{info}</div>
</div>

<style>
	.viewport { position: relative; width: 100%; height: 100%; }
	canvas { width: 100%; height: 100%; display: block; cursor: crosshair; }
	.info {
		position: absolute; bottom: 6px; left: 8px;
		font: 11px var(--font-mono); color: var(--text-muted);
		pointer-events: none;
	}
</style>

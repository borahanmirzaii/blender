<script lang="ts">
	import { onMount } from 'svelte';
	import { sceneStore } from '$lib/stores/scene.svelte';
	import { loadBlenderWasm } from '$lib/wasm/loader';

	let canvas: HTMLCanvasElement;
	let overlayInfo = $state('Initializing WebGPU...');
	let gpuAvailable = $state(false);

	// Camera orbit state
	let isDragging = $state(false);
	let isPanning = $state(false);
	let lastMouseX = 0;
	let lastMouseY = 0;
	let cameraAzimuth = $state(Math.PI / 4);
	let cameraElevation = $state(Math.PI / 6);
	let cameraDistance = $state(10);
	let cameraTarget = $state([0, 0, 0]);

	onMount(async () => {
		// Load WASM module
		const wasm = await loadBlenderWasm();
		await sceneStore.initFromWasm(wasm.scene);

		// Try to initialize WebGPU
		if (navigator.gpu) {
			try {
				const adapter = await navigator.gpu.requestAdapter();
				if (adapter) {
					const device = await adapter.requestDevice();
					gpuAvailable = true;
					overlayInfo = `WebGPU Ready | ${sceneStore.objects.length} objects`;
					startRenderLoop(device);
				} else {
					overlayInfo = 'No GPU adapter — using software fallback';
				}
			} catch (e) {
				overlayInfo = 'WebGPU init failed — using software fallback';
			}
		} else {
			overlayInfo = 'WebGPU not supported — using Canvas2D fallback';
			startFallbackRender();
		}
	});

	function startRenderLoop(device: GPUDevice) {
		const ctx = canvas.getContext('webgpu');
		if (!ctx) return;

		const format = navigator.gpu.getPreferredCanvasFormat();
		ctx.configure({ device, format, alphaMode: 'premultiplied' });

		function frame() {
			if (!canvas) return;

			// Resize if needed
			const dpr = window.devicePixelRatio || 1;
			const displayWidth = Math.floor(canvas.clientWidth * dpr);
			const displayHeight = Math.floor(canvas.clientHeight * dpr);
			if (canvas.width !== displayWidth || canvas.height !== displayHeight) {
				canvas.width = displayWidth;
				canvas.height = displayHeight;
			}

			// Render dark viewport background with grid
			const encoder = device.createCommandEncoder();
			const textureView = ctx.getCurrentTexture().createView();
			const pass = encoder.beginRenderPass({
				colorAttachments: [{
					view: textureView,
					clearValue: { r: 0.18, g: 0.18, b: 0.18, a: 1.0 },
					loadOp: 'clear',
					storeOp: 'store',
				}],
			});
			pass.end();
			device.queue.submit([encoder.finish()]);

			requestAnimationFrame(frame);
		}

		requestAnimationFrame(frame);
	}

	/** Canvas2D fallback when WebGPU is not available */
	function startFallbackRender() {
		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		function frame() {
			if (!canvas || !ctx) return;
			canvas.width = canvas.clientWidth * (window.devicePixelRatio || 1);
			canvas.height = canvas.clientHeight * (window.devicePixelRatio || 1);

			// Dark background
			ctx.fillStyle = '#2d2d2d';
			ctx.fillRect(0, 0, canvas.width, canvas.height);

			// Grid
			ctx.strokeStyle = '#3a3a3a';
			ctx.lineWidth = 1;
			const gridSize = 40;
			for (let x = 0; x < canvas.width; x += gridSize) {
				ctx.beginPath();
				ctx.moveTo(x, 0);
				ctx.lineTo(x, canvas.height);
				ctx.stroke();
			}
			for (let y = 0; y < canvas.height; y += gridSize) {
				ctx.beginPath();
				ctx.moveTo(0, y);
				ctx.lineTo(canvas.width, y);
				ctx.stroke();
			}

			// Center axes
			const cx = canvas.width / 2;
			const cy = canvas.height / 2;

			// X axis (red)
			ctx.strokeStyle = '#c44';
			ctx.lineWidth = 2;
			ctx.beginPath();
			ctx.moveTo(cx, cy);
			ctx.lineTo(cx + 80, cy);
			ctx.stroke();

			// Y axis (green)
			ctx.strokeStyle = '#4c8';
			ctx.beginPath();
			ctx.moveTo(cx, cy);
			ctx.lineTo(cx, cy - 80);
			ctx.stroke();

			// Draw objects as simple shapes
			const scene = sceneStore;
			for (const obj of scene.objects) {
				const ox = cx + obj.position[0] * gridSize;
				const oy = cy - obj.position[1] * gridSize;

				ctx.fillStyle = obj.selected ? '#4b8bbf' : '#888';
				ctx.strokeStyle = obj.selected ? '#5a9fd4' : '#666';
				ctx.lineWidth = obj.selected ? 2 : 1;

				if (obj.type === 'mesh') {
					ctx.fillRect(ox - 15, oy - 15, 30, 30);
					ctx.strokeRect(ox - 15, oy - 15, 30, 30);
				} else {
					ctx.beginPath();
					ctx.arc(ox, oy, 8, 0, Math.PI * 2);
					ctx.fill();
					ctx.stroke();
				}

				// Label
				ctx.fillStyle = '#ccc';
				ctx.font = '11px sans-serif';
				ctx.fillText(obj.name, ox + 18, oy + 4);
			}

			// Info text
			ctx.fillStyle = '#666';
			ctx.font = '11px monospace';
			ctx.fillText(`Objects: ${scene.objects.length} | Frame: ${scene.frameCurrent}`, 10, canvas.height - 10);

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
		lastMouseX = e.clientX;
		lastMouseY = e.clientY;
	}

	function onMouseMove(e: MouseEvent) {
		const dx = e.clientX - lastMouseX;
		const dy = e.clientY - lastMouseY;
		lastMouseX = e.clientX;
		lastMouseY = e.clientY;

		if (isDragging) {
			cameraAzimuth += dx * 0.01;
			cameraElevation = Math.max(-1.5, Math.min(1.5, cameraElevation + dy * 0.01));
		} else if (isPanning) {
			cameraTarget[0] -= dx * 0.02;
			cameraTarget[1] += dy * 0.02;
		}
	}

	function onMouseUp() {
		isDragging = false;
		isPanning = false;
	}

	function onWheel(e: WheelEvent) {
		e.preventDefault();
		cameraDistance *= 1 + e.deltaY * 0.001;
		cameraDistance = Math.max(0.5, Math.min(100, cameraDistance));
	}
</script>

<div class="viewport-container">
	<canvas
		bind:this={canvas}
		onmousedown={onMouseDown}
		onmousemove={onMouseMove}
		onmouseup={onMouseUp}
		onmouseleave={onMouseUp}
		onwheel={onWheel}
	></canvas>

	<!-- Viewport overlay -->
	<div class="viewport-overlay">
		<span class="overlay-info">{overlayInfo}</span>
		<div class="gizmo">
			<span class="axis-x">X</span>
			<span class="axis-y">Y</span>
			<span class="axis-z">Z</span>
		</div>
	</div>

	<!-- Viewport header -->
	<div class="viewport-header">
		<select>
			<option>Solid</option>
			<option>Wireframe</option>
			<option>Material Preview</option>
			<option>Rendered</option>
		</select>
	</div>
</div>

<style>
	.viewport-container {
		position: relative;
		width: 100%;
		height: 100%;
	}

	canvas {
		width: 100%;
		height: 100%;
		display: block;
		cursor: crosshair;
	}

	.viewport-overlay {
		position: absolute;
		bottom: 8px;
		left: 8px;
		right: 8px;
		display: flex;
		justify-content: space-between;
		pointer-events: none;
	}

	.overlay-info {
		font-size: 11px;
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	.gizmo {
		display: flex;
		gap: 6px;
		font-size: 11px;
		font-weight: bold;
	}

	.axis-x { color: #e44; }
	.axis-y { color: #4c8; }
	.axis-z { color: #48f; }

	.viewport-header {
		position: absolute;
		top: 6px;
		left: 6px;
	}

	.viewport-header select {
		font-size: 11px;
		padding: 2px 6px;
		background: rgba(30, 30, 30, 0.8);
		backdrop-filter: blur(4px);
	}
</style>

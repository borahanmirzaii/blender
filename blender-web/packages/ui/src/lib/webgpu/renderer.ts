/// WebGPU viewport renderer.
/// Manages the GPU device, render pipelines, and frame loop.

import { MESH_SHADER_WGSL } from './shaders';

export interface RendererState {
	device: GPUDevice;
	context: GPUCanvasContext;
	format: GPUTextureFormat;
	pipeline: GPURenderPipeline;
	depthTexture: GPUTexture;
}

/**
 * Initialize WebGPU and create the render pipeline.
 */
export async function initWebGPU(canvas: HTMLCanvasElement): Promise<RendererState | null> {
	if (!navigator.gpu) {
		console.error('WebGPU is not supported in this browser');
		return null;
	}

	const adapter = await navigator.gpu.requestAdapter({
		powerPreference: 'high-performance',
	});

	if (!adapter) {
		console.error('No suitable GPU adapter found');
		return null;
	}

	const device = await adapter.requestDevice({
		requiredFeatures: [],
		requiredLimits: {},
	});

	const context = canvas.getContext('webgpu');
	if (!context) {
		console.error('Failed to get WebGPU context');
		return null;
	}

	const format = navigator.gpu.getPreferredCanvasFormat();
	context.configure({
		device,
		format,
		alphaMode: 'premultiplied',
	});

	// Create shader module
	const shaderModule = device.createShaderModule({
		label: 'Mesh PBR Shader',
		code: MESH_SHADER_WGSL,
	});

	// Bind group layouts
	const uniformBindGroupLayout = device.createBindGroupLayout({
		entries: [{
			binding: 0,
			visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
			buffer: { type: 'uniform' },
		}],
	});

	const pipelineLayout = device.createPipelineLayout({
		bindGroupLayouts: [uniformBindGroupLayout, uniformBindGroupLayout, uniformBindGroupLayout],
	});

	// Render pipeline
	const pipeline = device.createRenderPipeline({
		label: 'Mesh Pipeline',
		layout: pipelineLayout,
		vertex: {
			module: shaderModule,
			entryPoint: 'vs_main',
			buffers: [{
				arrayStride: 32, // 3*f32 + 3*f32 + 2*f32
				attributes: [
					{ shaderLocation: 0, offset: 0, format: 'float32x3' },  // position
					{ shaderLocation: 1, offset: 12, format: 'float32x3' }, // normal
					{ shaderLocation: 2, offset: 24, format: 'float32x2' }, // uv
				],
			}],
		},
		fragment: {
			module: shaderModule,
			entryPoint: 'fs_main',
			targets: [{ format }],
		},
		primitive: {
			topology: 'triangle-list',
			cullMode: 'back',
		},
		depthStencil: {
			depthWriteEnabled: true,
			depthCompare: 'less',
			format: 'depth24plus',
		},
	});

	// Depth texture
	const depthTexture = device.createTexture({
		size: [canvas.width, canvas.height],
		format: 'depth24plus',
		usage: GPUTextureUsage.RENDER_ATTACHMENT,
	});

	console.log('[WebGPU] Renderer initialized');

	return { device, context, format, pipeline, depthTexture };
}

/**
 * Resize the depth texture when canvas size changes.
 */
export function resizeDepthTexture(state: RendererState, width: number, height: number): GPUTexture {
	state.depthTexture.destroy();
	state.depthTexture = state.device.createTexture({
		size: [width, height],
		format: 'depth24plus',
		usage: GPUTextureUsage.RENDER_ATTACHMENT,
	});
	return state.depthTexture;
}

/**
 * Render a single frame.
 */
export function renderFrame(
	state: RendererState,
	uniformBindGroup: GPUBindGroup,
	materialBindGroup: GPUBindGroup,
	lightBindGroup: GPUBindGroup,
	vertexBuffer: GPUBuffer,
	indexBuffer: GPUBuffer,
	indexCount: number,
) {
	const commandEncoder = state.device.createCommandEncoder();
	const textureView = state.context.getCurrentTexture().createView();

	const renderPass = commandEncoder.beginRenderPass({
		colorAttachments: [{
			view: textureView,
			clearValue: { r: 0.18, g: 0.18, b: 0.18, a: 1.0 },
			loadOp: 'clear',
			storeOp: 'store',
		}],
		depthStencilAttachment: {
			view: state.depthTexture.createView(),
			depthClearValue: 1.0,
			depthLoadOp: 'clear',
			depthStoreOp: 'store',
		},
	});

	renderPass.setPipeline(state.pipeline);
	renderPass.setBindGroup(0, uniformBindGroup);
	renderPass.setBindGroup(1, materialBindGroup);
	renderPass.setBindGroup(2, lightBindGroup);
	renderPass.setVertexBuffer(0, vertexBuffer);
	renderPass.setIndexBuffer(indexBuffer, 'uint32');
	renderPass.drawIndexed(indexCount);
	renderPass.end();

	state.device.queue.submit([commandEncoder.finish()]);
}

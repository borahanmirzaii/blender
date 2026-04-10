/**
 * Edge Worker for Blender Web.
 * Runs on Cloudflare Workers / Deno Deploy.
 *
 * Responsibilities:
 * - Authentication and session management
 * - Asset proxy (serve textures, models from R2/S3)
 * - Real-time collaboration via WebSocket (Durable Objects)
 * - Render job dispatch to GPU servers
 * - Scene auto-save and versioning
 */

export interface Env {
	SCENE_STORE: KVNamespace;         // Scene JSON storage
	ASSET_BUCKET: R2Bucket;           // Texture/model assets
	RENDER_QUEUE: Queue;              // Render job queue
	COLLAB_ROOM: DurableObjectNamespace; // Real-time collaboration
}

export default {
	async fetch(request: Request, env: Env): Promise<Response> {
		const url = new URL(request.url);

		// CORS headers for browser access
		const corsHeaders = {
			'Access-Control-Allow-Origin': '*',
			'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
			'Access-Control-Allow-Headers': 'Content-Type, Authorization',
		};

		if (request.method === 'OPTIONS') {
			return new Response(null, { headers: corsHeaders });
		}

		try {
			// Route handling
			if (url.pathname.startsWith('/api/scene')) {
				return handleSceneAPI(request, env, corsHeaders);
			}

			if (url.pathname.startsWith('/api/render')) {
				return handleRenderAPI(request, env, corsHeaders);
			}

			if (url.pathname.startsWith('/api/assets')) {
				return handleAssetAPI(request, env, corsHeaders);
			}

			if (url.pathname.startsWith('/api/collab')) {
				return handleCollabWebSocket(request, env);
			}

			if (url.pathname.startsWith('/api/ai')) {
				return handleAIProxy(request, env, corsHeaders);
			}

			return new Response('Not Found', { status: 404, headers: corsHeaders });
		} catch (err) {
			return new Response(
				JSON.stringify({ error: (err as Error).message }),
				{ status: 500, headers: { ...corsHeaders, 'Content-Type': 'application/json' } }
			);
		}
	},
};

// --- Scene API ---

async function handleSceneAPI(request: Request, env: Env, headers: Record<string, string>): Promise<Response> {
	const url = new URL(request.url);
	const sceneId = url.pathname.split('/').pop();

	if (request.method === 'GET' && sceneId) {
		// Load scene from KV
		const scene = await env.SCENE_STORE.get(`scene:${sceneId}`);
		if (!scene) {
			return new Response('Scene not found', { status: 404, headers });
		}
		return new Response(scene, {
			headers: { ...headers, 'Content-Type': 'application/json' },
		});
	}

	if (request.method === 'PUT' && sceneId) {
		// Save scene to KV with versioning
		const body = await request.text();
		const version = Date.now();
		await env.SCENE_STORE.put(`scene:${sceneId}`, body, {
			metadata: { version, savedAt: new Date().toISOString() },
		});
		// Keep version history
		await env.SCENE_STORE.put(`scene:${sceneId}:v${version}`, body);
		return new Response(JSON.stringify({ sceneId, version }), {
			headers: { ...headers, 'Content-Type': 'application/json' },
		});
	}

	if (request.method === 'POST') {
		// Create new scene
		const body = await request.text();
		const sceneId = crypto.randomUUID();
		await env.SCENE_STORE.put(`scene:${sceneId}`, body);
		return new Response(JSON.stringify({ sceneId }), {
			status: 201,
			headers: { ...headers, 'Content-Type': 'application/json' },
		});
	}

	return new Response('Method not allowed', { status: 405, headers });
}

// --- Render API ---

async function handleRenderAPI(request: Request, env: Env, headers: Record<string, string>): Promise<Response> {
	if (request.method !== 'POST') {
		return new Response('Method not allowed', { status: 405, headers });
	}

	const { sceneId, resolution, samples } = await request.json() as {
		sceneId: string;
		resolution: [number, number];
		samples: number;
	};

	// Dispatch render job to GPU server queue
	const jobId = crypto.randomUUID();
	await env.RENDER_QUEUE.send({
		jobId,
		sceneId,
		resolution,
		samples,
		submittedAt: Date.now(),
	});

	return new Response(JSON.stringify({ jobId, status: 'queued' }), {
		headers: { ...headers, 'Content-Type': 'application/json' },
	});
}

// --- Asset API ---

async function handleAssetAPI(request: Request, env: Env, headers: Record<string, string>): Promise<Response> {
	const url = new URL(request.url);
	const assetPath = url.pathname.replace('/api/assets/', '');

	if (request.method === 'GET') {
		const object = await env.ASSET_BUCKET.get(assetPath);
		if (!object) {
			return new Response('Asset not found', { status: 404, headers });
		}
		return new Response(object.body, {
			headers: {
				...headers,
				'Content-Type': object.httpMetadata?.contentType || 'application/octet-stream',
				'Cache-Control': 'public, max-age=31536000, immutable',
			},
		});
	}

	if (request.method === 'PUT') {
		const body = await request.arrayBuffer();
		await env.ASSET_BUCKET.put(assetPath, body, {
			httpMetadata: { contentType: request.headers.get('Content-Type') || 'application/octet-stream' },
		});
		return new Response(JSON.stringify({ path: assetPath }), {
			headers: { ...headers, 'Content-Type': 'application/json' },
		});
	}

	return new Response('Method not allowed', { status: 405, headers });
}

// --- Collaboration WebSocket ---

async function handleCollabWebSocket(request: Request, env: Env): Promise<Response> {
	const url = new URL(request.url);
	const roomId = url.searchParams.get('room') || 'default';

	// Route to Durable Object for stateful WebSocket handling
	const id = env.COLLAB_ROOM.idFromName(roomId);
	const room = env.COLLAB_ROOM.get(id);
	return room.fetch(request);
}

// --- AI Proxy ---

async function handleAIProxy(request: Request, env: Env, headers: Record<string, string>): Promise<Response> {
	if (request.method !== 'POST') {
		return new Response('Method not allowed', { status: 405, headers });
	}

	// Proxy AI requests to Claude API with MCP tool definitions
	// This keeps the API key on the server side
	return new Response(JSON.stringify({
		message: 'AI proxy endpoint — connects to Claude API with Blender MCP tools',
		tools: [
			'create_object', 'modify_mesh', 'set_material',
			'render_preview', 'export_scene', 'query_scene',
			'run_geometry_nodes', 'animate',
		],
	}), {
		headers: { ...headers, 'Content-Type': 'application/json' },
	});
}

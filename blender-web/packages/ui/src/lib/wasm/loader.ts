/**
 * WASM loader — initializes the blender-wasm module from wasm-pack output.
 * The WASM binary is served from /wasm/ as a static asset.
 */

// Import the wasm-pack generated JS glue.
// In dev mode, this loads from static/wasm/.
// At build time, these are bundled or served statically.
import init, { SceneHandle } from '/wasm/blender_wasm.js';

let initialized = false;

/**
 * Initialize the WASM module. Call once on app startup.
 * Returns the SceneHandle class for creating scenes.
 */
export async function initWasm(): Promise<typeof SceneHandle> {
	if (!initialized) {
		await init('/wasm/blender_wasm_bg.wasm');
		initialized = true;
		console.log('[blender-wasm] Module loaded (586KB)');
	}
	return SceneHandle;
}

export { SceneHandle };

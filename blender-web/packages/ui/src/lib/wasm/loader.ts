/// WASM module loader — initializes blender-wasm and WASI polyfill.
///
/// Two loading strategies:
/// 1. Browser: wasm-bindgen generated JS glue (wasm32-unknown-unknown)
/// 2. WASI:    wasmtime/browser-wasi-shim for file access (wasm32-wasip1)

export interface WasmModules {
	scene: any;     // SceneHandle class from blender-wasm
	wasi: any;      // WASI filesystem module (optional)
}

/**
 * Load the blender-wasm module compiled by wasm-pack.
 * In production, this loads from the CDN/static assets.
 * In dev, it loads from the wasm-pack output directory.
 */
export async function loadBlenderWasm(): Promise<WasmModules> {
	try {
		// Dynamic import of wasm-pack generated module
		// wasm-pack outputs: blender_wasm.js + blender_wasm_bg.wasm
		const wasm = await import(
			/* @vite-ignore */
			'/wasm/blender_wasm.js'
		);

		// wasm-bindgen init function loads the .wasm binary
		await wasm.default();

		console.log('[blender-wasm] WASM module loaded successfully');

		return {
			scene: wasm.SceneHandle,
			wasi: null,
		};
	} catch (err) {
		console.warn('[blender-wasm] WASM not available, using mock:', err);
		return {
			scene: createMockSceneHandle(),
			wasi: null,
		};
	}
}

/**
 * Load WASI module for local file access.
 * Uses the File System Access API to grant directory access,
 * then polyfills WASI fd_* calls to route through it.
 */
export async function initWasiFileAccess(): Promise<FileSystemDirectoryHandle | null> {
	if (!('showDirectoryPicker' in window)) {
		console.warn('[WASI] File System Access API not available');
		return null;
	}

	try {
		// Prompt user to select a project directory
		const dirHandle = await (window as any).showDirectoryPicker({
			mode: 'readwrite',
		});

		console.log(`[WASI] Granted access to directory: ${dirHandle.name}`);
		return dirHandle;
	} catch (err) {
		console.warn('[WASI] User cancelled directory picker:', err);
		return null;
	}
}

/**
 * Read a file from a WASI-granted directory handle.
 */
export async function readFileFromHandle(
	dirHandle: FileSystemDirectoryHandle,
	path: string
): Promise<Uint8Array> {
	const parts = path.split('/').filter(Boolean);
	let current: FileSystemDirectoryHandle = dirHandle;

	// Navigate to subdirectory
	for (const part of parts.slice(0, -1)) {
		current = await current.getDirectoryHandle(part);
	}

	const fileName = parts[parts.length - 1];
	const fileHandle = await current.getFileHandle(fileName);
	const file = await fileHandle.getFile();
	const buffer = await file.arrayBuffer();
	return new Uint8Array(buffer);
}

/**
 * Write a file to a WASI-granted directory handle.
 */
export async function writeFileToHandle(
	dirHandle: FileSystemDirectoryHandle,
	path: string,
	data: Uint8Array
): Promise<void> {
	const parts = path.split('/').filter(Boolean);
	let current: FileSystemDirectoryHandle = dirHandle;

	// Create subdirectories as needed
	for (const part of parts.slice(0, -1)) {
		current = await current.getDirectoryHandle(part, { create: true });
	}

	const fileName = parts[parts.length - 1];
	const fileHandle = await current.getFileHandle(fileName, { create: true });
	const writable = await (fileHandle as any).createWritable();
	await writable.write(data);
	await writable.close();
}

/**
 * Mock SceneHandle for development without WASM compilation.
 */
function createMockSceneHandle() {
	return class MockSceneHandle {
		private objects: { name: string; position: number[]; scale: number[] }[] = [];
		private meshCount_ = 0;

		static defaultScene() {
			const s = new MockSceneHandle();
			s.objects = [
				{ name: 'Cube', position: [0, 0, 0], scale: [1, 1, 1] },
				{ name: 'Light', position: [4, 4, 3], scale: [1, 1, 1] },
				{ name: 'Camera', position: [7, -6, 5], scale: [1, 1, 1] },
			];
			s.meshCount_ = 1;
			return s;
		}

		objectCount() { return this.objects.length; }
		meshCount() { return this.meshCount_; }
		objectName(i: number) { return this.objects[i]?.name ?? 'Unknown'; }

		addPrimitive(type: string, name: string) {
			this.objects.push({ name, position: [0, 0, 0], scale: [1, 1, 1] });
			this.meshCount_++;
			return this.objects.length - 1;
		}

		setPosition(i: number, x: number, y: number, z: number) {
			if (this.objects[i]) this.objects[i].position = [x, y, z];
		}

		setScale(i: number, x: number, y: number, z: number) {
			if (this.objects[i]) this.objects[i].scale = [x, y, z];
		}

		deleteObject(i: number) { this.objects.splice(i, 1); }

		toJson() { return JSON.stringify({ objects: this.objects }, null, 2); }
	};
}

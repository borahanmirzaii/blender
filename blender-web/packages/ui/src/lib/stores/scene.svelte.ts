/**
 * Scene store — bridges WASM SceneHandle with Svelte 5 reactivity.
 *
 * The WASM module owns the actual scene data (Rust memory).
 * This store mirrors a lightweight JS representation for UI binding.
 */

import type { SceneHandle } from '/wasm/blender_wasm.js';

export interface SceneObject {
	index: number;
	name: string;
	type: string;
	visible: boolean;
	meshIndex: number;
	materialIndex: number;
}

function createSceneStore() {
	let objects = $state<SceneObject[]>([]);
	let activeIndex = $state<number | null>(null);
	let wasmReady = $state(false);
	let handle: SceneHandle | null = null;

	function syncFromWasm() {
		if (!handle) return;
		const count = handle.objectCount();
		const objs: SceneObject[] = [];
		for (let i = 0; i < count; i++) {
			objs.push({
				index: i,
				name: handle.objectName(i),
				type: handle.objectType(i),
				visible: handle.objectVisible(i),
				meshIndex: handle.objectMeshIndex(i),
				materialIndex: handle.objectMaterialIndex(i),
			});
		}
		objects = objs;
	}

	return {
		get objects() { return objects; },
		get activeIndex() { return activeIndex; },
		get activeObject() {
			return activeIndex !== null ? objects[activeIndex] ?? null : null;
		},
		get wasmReady() { return wasmReady; },
		get handle() { return handle; },

		init(h: SceneHandle) {
			handle = h;
			wasmReady = true;
			syncFromWasm();
		},

		selectObject(idx: number | null) {
			activeIndex = idx;
		},

		addPrimitive(kind: string, name: string): number | null {
			if (!handle) return null;
			const idx = handle.addPrimitive(kind, name);
			activeIndex = idx;
			syncFromWasm();
			return idx;
		},

		deleteObject(idx: number) {
			if (!handle) return;
			handle.deleteObject(idx);
			if (activeIndex === idx) activeIndex = null;
			syncFromWasm();
		},

		setPosition(idx: number, x: number, y: number, z: number) {
			handle?.setPosition(idx, x, y, z);
		},

		exportJson(): string | null {
			return handle?.toJson() ?? null;
		},

		sync() { syncFromWasm(); },
	};
}

export const sceneStore = createSceneStore();

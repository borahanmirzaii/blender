/// Reactive scene state — bridges WASM SceneHandle with Svelte's reactivity.
/// Uses Svelte 5 runes ($state, $derived) for fine-grained reactivity.

export interface SceneObject {
	index: number;
	name: string;
	type: 'mesh' | 'light' | 'camera' | 'empty';
	visible: boolean;
	selected: boolean;
	position: [number, number, number];
	scale: [number, number, number];
}

export interface SceneMaterial {
	index: number;
	name: string;
	baseColor: [number, number, number];
	metallic: number;
	roughness: number;
}

interface SceneState {
	objects: SceneObject[];
	materials: SceneMaterial[];
	activeObjectIndex: number | null;
	frameCurrent: number;
	frameStart: number;
	frameEnd: number;
	fps: number;
	wasmReady: boolean;
}

function createSceneStore() {
	let state = $state<SceneState>({
		objects: [],
		materials: [],
		activeObjectIndex: null,
		frameCurrent: 1,
		frameStart: 1,
		frameEnd: 250,
		fps: 24,
		wasmReady: false,
	});

	// WASM handle reference (not reactive — opaque pointer)
	let wasmScene: any = null;

	return {
		get objects() { return state.objects; },
		get materials() { return state.materials; },
		get activeObjectIndex() { return state.activeObjectIndex; },
		get activeObject() {
			if (state.activeObjectIndex === null) return null;
			return state.objects[state.activeObjectIndex] ?? null;
		},
		get frameCurrent() { return state.frameCurrent; },
		get frameStart() { return state.frameStart; },
		get frameEnd() { return state.frameEnd; },
		get fps() { return state.fps; },
		get wasmReady() { return state.wasmReady; },

		/** Initialize from WASM default scene */
		async initFromWasm(SceneHandle: any) {
			wasmScene = SceneHandle.defaultScene();
			state.wasmReady = true;
			this.syncFromWasm();
		},

		/** Sync JS state from WASM scene (call after mutations) */
		syncFromWasm() {
			if (!wasmScene) return;
			const count = wasmScene.objectCount();
			const objects: SceneObject[] = [];
			for (let i = 0; i < count; i++) {
				objects.push({
					index: i,
					name: wasmScene.objectName(i),
					type: 'mesh', // simplified — full impl would check data type
					visible: true,
					selected: i === state.activeObjectIndex,
					position: [0, 0, 0],
					scale: [1, 1, 1],
				});
			}
			state.objects = objects;
		},

		/** Add a primitive to the scene */
		addPrimitive(type: string, name: string) {
			if (!wasmScene) return;
			const idx = wasmScene.addPrimitive(type, name);
			state.activeObjectIndex = idx;
			this.syncFromWasm();
		},

		/** Select an object */
		selectObject(index: number | null) {
			state.activeObjectIndex = index;
			state.objects = state.objects.map((obj, i) => ({
				...obj,
				selected: i === index,
			}));
		},

		/** Delete the active object */
		deleteActive() {
			if (state.activeObjectIndex === null || !wasmScene) return;
			wasmScene.deleteObject(state.activeObjectIndex);
			state.activeObjectIndex = null;
			this.syncFromWasm();
		},

		/** Set object position */
		setPosition(index: number, x: number, y: number, z: number) {
			if (!wasmScene) return;
			wasmScene.setPosition(index, x, y, z);
			if (state.objects[index]) {
				state.objects[index].position = [x, y, z];
			}
		},

		/** Set animation frame */
		setFrame(frame: number) {
			state.frameCurrent = Math.max(state.frameStart, Math.min(state.frameEnd, frame));
		},

		/** Get WASM scene handle for direct GPU operations */
		getWasmScene() { return wasmScene; },

		/** Export scene as JSON */
		exportJson(): string | null {
			if (!wasmScene) return null;
			return wasmScene.toJson();
		},
	};
}

export const sceneStore = createSceneStore();

/**
 * Scene manager — in-memory scene state that MCP tools operate on.
 * In production, this connects to the WASM runtime or the edge worker.
 */

export interface SceneObject {
	name: string;
	type: 'mesh' | 'light' | 'camera' | 'empty';
	meshType?: string;
	position: [number, number, number];
	rotation: [number, number, number];
	scale: [number, number, number];
	material?: MaterialData;
	parent?: string;
	visible: boolean;
}

export interface MaterialData {
	name: string;
	baseColor: [number, number, number];
	metallic: number;
	roughness: number;
	emissive: [number, number, number];
}

export interface KeyframeData {
	object: string;
	property: string;
	frame: number;
	value: number[];
}

export class BlenderSceneManager {
	private objects: Map<string, SceneObject> = new Map();
	private keyframes: KeyframeData[] = [];
	private nextId = 1;

	constructor() {
		// Initialize with default scene
		this.addObject({
			name: 'Cube',
			type: 'mesh',
			meshType: 'cube',
			position: [0, 0, 0],
			rotation: [0, 0, 0],
			scale: [1, 1, 1],
			material: {
				name: 'Default',
				baseColor: [0.8, 0.8, 0.8],
				metallic: 0,
				roughness: 0.5,
				emissive: [0, 0, 0],
			},
			visible: true,
		});

		this.addObject({
			name: 'Light',
			type: 'light',
			position: [4, 4, 3],
			rotation: [0, 0, 0],
			scale: [1, 1, 1],
			visible: true,
		});

		this.addObject({
			name: 'Camera',
			type: 'camera',
			position: [7, -6, 5],
			rotation: [1.1, 0, 0.8],
			scale: [1, 1, 1],
			visible: true,
		});
	}

	addObject(obj: SceneObject): string {
		// Ensure unique name
		let name = obj.name;
		if (this.objects.has(name)) {
			name = `${obj.name}.${String(this.nextId++).padStart(3, '0')}`;
			obj.name = name;
		}
		this.objects.set(name, obj);
		return name;
	}

	getObject(name: string): SceneObject | undefined {
		return this.objects.get(name);
	}

	removeObject(name: string): boolean {
		return this.objects.delete(name);
	}

	listObjects(): SceneObject[] {
		return Array.from(this.objects.values());
	}

	findObjects(query: { type?: string; nameContains?: string }): SceneObject[] {
		let results = Array.from(this.objects.values());
		if (query.type) {
			results = results.filter(o => o.type === query.type);
		}
		if (query.nameContains) {
			const lower = query.nameContains.toLowerCase();
			results = results.filter(o => o.name.toLowerCase().includes(lower));
		}
		return results;
	}

	setTransform(
		name: string,
		transform: { position?: [number, number, number]; rotation?: [number, number, number]; scale?: [number, number, number] }
	): boolean {
		const obj = this.objects.get(name);
		if (!obj) return false;
		if (transform.position) obj.position = transform.position;
		if (transform.rotation) obj.rotation = transform.rotation;
		if (transform.scale) obj.scale = transform.scale;
		return true;
	}

	setMaterial(name: string, material: Partial<MaterialData>): boolean {
		const obj = this.objects.get(name);
		if (!obj) return false;
		obj.material = {
			name: material.name ?? obj.material?.name ?? 'Material',
			baseColor: material.baseColor ?? obj.material?.baseColor ?? [0.8, 0.8, 0.8],
			metallic: material.metallic ?? obj.material?.metallic ?? 0,
			roughness: material.roughness ?? obj.material?.roughness ?? 0.5,
			emissive: material.emissive ?? obj.material?.emissive ?? [0, 0, 0],
		};
		return true;
	}

	addKeyframe(data: KeyframeData): void {
		// Remove existing keyframe at same frame/property
		this.keyframes = this.keyframes.filter(
			k => !(k.object === data.object && k.property === data.property && k.frame === data.frame)
		);
		this.keyframes.push(data);
	}

	getKeyframes(objectName: string): KeyframeData[] {
		return this.keyframes.filter(k => k.object === objectName);
	}

	toJSON(): object {
		return {
			objects: Array.from(this.objects.entries()).map(([name, obj]) => ({ ...obj, name })),
			keyframes: this.keyframes,
		};
	}

	objectCount(): number {
		return this.objects.size;
	}
}

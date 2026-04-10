/**
 * MCP Tool registration — each tool maps to a Blender operation.
 * These tools are callable by Claude and other MCP-compatible AI models.
 */

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import { BlenderSceneManager } from './scene-manager.js';

export function registerTools(server: McpServer, scene: BlenderSceneManager) {

	// --- create_object ---
	server.tool(
		'create_object',
		'Add a new object to the Blender scene. Supports mesh primitives (cube, sphere, cylinder, plane, torus), lights (point, sun, spot, area), cameras, and empties.',
		{
			type: z.enum(['mesh', 'light', 'camera', 'empty']).describe('Object type'),
			name: z.string().describe('Object name'),
			meshType: z.enum(['cube', 'sphere', 'cylinder', 'plane', 'torus', 'cone', 'monkey'])
				.optional().describe('Mesh primitive type (required if type is mesh)'),
			position: z.tuple([z.number(), z.number(), z.number()])
				.optional().default([0, 0, 0]).describe('XYZ position'),
			scale: z.tuple([z.number(), z.number(), z.number()])
				.optional().default([1, 1, 1]).describe('XYZ scale'),
		},
		async ({ type, name, meshType, position, scale }) => {
			const objName = scene.addObject({
				name,
				type,
				meshType,
				position: position as [number, number, number],
				rotation: [0, 0, 0],
				scale: scale as [number, number, number],
				visible: true,
			});

			return {
				content: [{
					type: 'text',
					text: `Created ${type} "${objName}" at position [${position}] with scale [${scale}]. Scene now has ${scene.objectCount()} objects.`,
				}],
			};
		}
	);

	// --- modify_object ---
	server.tool(
		'modify_object',
		'Modify an existing object\'s transform (position, rotation, scale).',
		{
			name: z.string().describe('Object name to modify'),
			position: z.tuple([z.number(), z.number(), z.number()])
				.optional().describe('New XYZ position'),
			rotation: z.tuple([z.number(), z.number(), z.number()])
				.optional().describe('New XYZ rotation in radians'),
			scale: z.tuple([z.number(), z.number(), z.number()])
				.optional().describe('New XYZ scale'),
		},
		async ({ name, position, rotation, scale }) => {
			const success = scene.setTransform(name, {
				position: position as [number, number, number] | undefined,
				rotation: rotation as [number, number, number] | undefined,
				scale: scale as [number, number, number] | undefined,
			});

			if (!success) {
				return {
					content: [{ type: 'text', text: `Object "${name}" not found.` }],
					isError: true,
				};
			}

			const obj = scene.getObject(name)!;
			return {
				content: [{
					type: 'text',
					text: `Modified "${name}": position=[${obj.position}], rotation=[${obj.rotation}], scale=[${obj.scale}]`,
				}],
			};
		}
	);

	// --- set_material ---
	server.tool(
		'set_material',
		'Set PBR material properties on an object. Uses metallic-roughness workflow.',
		{
			objectName: z.string().describe('Object to apply material to'),
			materialName: z.string().optional().describe('Material name'),
			baseColor: z.tuple([z.number(), z.number(), z.number()])
				.optional().describe('RGB base color (0-1 range)'),
			metallic: z.number().min(0).max(1).optional().describe('Metallic factor (0=dielectric, 1=metal)'),
			roughness: z.number().min(0).max(1).optional().describe('Roughness factor (0=smooth, 1=rough)'),
			emissive: z.tuple([z.number(), z.number(), z.number()])
				.optional().describe('RGB emissive color'),
		},
		async ({ objectName, materialName, baseColor, metallic, roughness, emissive }) => {
			const success = scene.setMaterial(objectName, {
				name: materialName,
				baseColor: baseColor as [number, number, number] | undefined,
				metallic,
				roughness,
				emissive: emissive as [number, number, number] | undefined,
			});

			if (!success) {
				return {
					content: [{ type: 'text', text: `Object "${objectName}" not found.` }],
					isError: true,
				};
			}

			return {
				content: [{
					type: 'text',
					text: `Material set on "${objectName}": color=[${baseColor ?? 'unchanged'}], metallic=${metallic ?? 'unchanged'}, roughness=${roughness ?? 'unchanged'}`,
				}],
			};
		}
	);

	// --- delete_object ---
	server.tool(
		'delete_object',
		'Remove an object from the scene.',
		{
			name: z.string().describe('Name of object to delete'),
		},
		async ({ name }) => {
			const removed = scene.removeObject(name);
			return {
				content: [{
					type: 'text',
					text: removed
						? `Deleted "${name}". Scene now has ${scene.objectCount()} objects.`
						: `Object "${name}" not found.`,
				}],
				isError: !removed,
			};
		}
	);

	// --- query_scene ---
	server.tool(
		'query_scene',
		'Query the scene for objects. Can filter by type or name. Returns full object details.',
		{
			type: z.enum(['mesh', 'light', 'camera', 'empty']).optional().describe('Filter by object type'),
			nameContains: z.string().optional().describe('Filter by name substring'),
		},
		async ({ type, nameContains }) => {
			const results = scene.findObjects({ type, nameContains });
			return {
				content: [{
					type: 'text',
					text: `Found ${results.length} objects:\n${results.map(o =>
						`- ${o.name} (${o.type}${o.meshType ? ':' + o.meshType : ''}) at [${o.position}]`
					).join('\n')}`,
				}],
			};
		}
	);

	// --- export_scene ---
	server.tool(
		'export_scene',
		'Export the entire scene as JSON. Useful for saving, sending to renderers, or debugging.',
		{},
		async () => {
			const json = JSON.stringify(scene.toJSON(), null, 2);
			return {
				content: [{
					type: 'text',
					text: json,
				}],
			};
		}
	);

	// --- animate ---
	server.tool(
		'animate',
		'Set a keyframe on an object property at a specific frame.',
		{
			objectName: z.string().describe('Object to animate'),
			property: z.enum(['position', 'rotation', 'scale']).describe('Property to keyframe'),
			frame: z.number().int().describe('Frame number'),
			value: z.array(z.number()).length(3).describe('XYZ value at this frame'),
		},
		async ({ objectName, property, frame, value }) => {
			const obj = scene.getObject(objectName);
			if (!obj) {
				return {
					content: [{ type: 'text', text: `Object "${objectName}" not found.` }],
					isError: true,
				};
			}

			scene.addKeyframe({ object: objectName, property, frame, value });

			const keyframes = scene.getKeyframes(objectName);
			return {
				content: [{
					type: 'text',
					text: `Keyframe set on "${objectName}".${property} at frame ${frame} = [${value}]. Object has ${keyframes.length} keyframes total.`,
				}],
			};
		}
	);

	// --- duplicate_object ---
	server.tool(
		'duplicate_object',
		'Duplicate an existing object with an optional position offset.',
		{
			sourceName: z.string().describe('Name of object to duplicate'),
			newName: z.string().optional().describe('Name for the duplicate'),
			offset: z.tuple([z.number(), z.number(), z.number()])
				.optional().default([1, 0, 0]).describe('Position offset from original'),
		},
		async ({ sourceName, newName, offset }) => {
			const source = scene.getObject(sourceName);
			if (!source) {
				return {
					content: [{ type: 'text', text: `Object "${sourceName}" not found.` }],
					isError: true,
				};
			}

			const dupName = scene.addObject({
				...structuredClone(source),
				name: newName || `${sourceName}.copy`,
				position: [
					source.position[0] + (offset as [number, number, number])[0],
					source.position[1] + (offset as [number, number, number])[1],
					source.position[2] + (offset as [number, number, number])[2],
				],
			});

			return {
				content: [{
					type: 'text',
					text: `Duplicated "${sourceName}" as "${dupName}" at offset [${offset}].`,
				}],
			};
		}
	);
}

/**
 * MCP Prompt/Resource registration for AI agent workflows.
 * These provide context and instructions for Claude when working with Blender.
 */

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';

export function registerAgentPrompts(server: McpServer) {
	// --- Scene Builder Agent Prompt ---
	server.prompt(
		'scene-builder',
		'Build a 3D scene from a text description. Plans the objects, materials, and layout, then creates them step by step.',
		{ description: { description: 'Text description of the scene to build' } },
		({ description }) => ({
			messages: [{
				role: 'user',
				content: {
					type: 'text',
					text: `You are a 3D scene builder agent for Blender Web. Your job is to create a complete 3D scene based on the following description:

"${description}"

Plan your approach:
1. List all objects needed (meshes, lights, cameras)
2. Determine positions and scales for proper layout
3. Choose appropriate materials (PBR metallic-roughness)
4. Set up lighting for the mood described
5. Position the camera for the best composition

Use the available MCP tools to create each element:
- create_object: Add meshes, lights, cameras
- set_material: Apply PBR materials with colors
- modify_object: Adjust transforms
- animate: Add keyframes if motion is described

Work systematically — create objects one at a time, position them, then apply materials. Think about spatial relationships and scale.

Start building now.`
				}
			}]
		})
	);

	// --- Material Designer Agent Prompt ---
	server.prompt(
		'material-designer',
		'Design and apply PBR materials to objects based on a description.',
		{
			description: { description: 'Description of the desired material look' },
			objectName: { description: 'Name of the object to apply the material to' },
		},
		({ description, objectName }) => ({
			messages: [{
				role: 'user',
				content: {
					type: 'text',
					text: `You are a PBR material designer. Create a material for the object "${objectName}" based on this description:

"${description}"

Consider:
- Base color: What RGB values (0-1) match the described surface?
- Metallic: Is this a metal (1.0) or non-metal (0.0)? Mixed materials can use values in between.
- Roughness: Smooth/glossy (0.0-0.3), semi-rough (0.3-0.7), or rough/matte (0.7-1.0)?
- Emissive: Does this surface emit light? If so, what color?

Use the set_material tool to apply your designed material.`
				}
			}]
		})
	);

	// --- Scene Analyzer Agent Prompt ---
	server.prompt(
		'scene-analyzer',
		'Analyze the current scene and provide suggestions for improvement.',
		{},
		() => ({
			messages: [{
				role: 'user',
				content: {
					type: 'text',
					text: `You are a 3D scene analysis agent. Use the query_scene tool to examine the current scene, then provide:

1. **Scene Summary**: What objects exist, their types and positions
2. **Composition Analysis**: Is the layout balanced? Are objects at reasonable scales?
3. **Lighting Assessment**: Are there enough lights? Is the scene well-lit?
4. **Material Review**: Do materials look appropriate for the objects?
5. **Suggestions**: What could be added or changed to improve the scene?

Start by querying the scene, then provide your analysis.`
				}
			}]
		})
	);

	// --- Animation Agent Prompt ---
	server.prompt(
		'animator',
		'Create animations for objects in the scene.',
		{
			description: { description: 'Description of the desired animation' },
			frameRange: { description: 'Frame range (e.g., "1-120")' },
		},
		({ description, frameRange }) => ({
			messages: [{
				role: 'user',
				content: {
					type: 'text',
					text: `You are an animation agent for Blender Web. Create animations based on this description:

"${description}"

Frame range: ${frameRange}

Plan your keyframes:
1. Identify which objects need to move
2. Break the motion into key poses
3. Set keyframes at appropriate frames
4. Consider easing (the interpolation between keyframes is automatic)

Use the animate tool to set keyframes. Remember:
- Position keyframes: [x, y, z]
- Rotation keyframes: [x, y, z] in radians
- Scale keyframes: [x, y, z]

Create smooth, natural-looking motion by spacing keyframes appropriately.`
				}
			}]
		})
	);
}

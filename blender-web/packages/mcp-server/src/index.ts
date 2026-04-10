/**
 * Blender Web MCP Server
 *
 * Exposes Blender scene operations as MCP tools that AI agents (Claude)
 * can invoke to create, modify, and query 3D scenes.
 *
 * Runs as a stdio MCP server — connect via Claude Desktop, Claude Code,
 * or any MCP-compatible client.
 */

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { z } from 'zod';
import { BlenderSceneManager } from './tools/scene-manager.js';
import { registerTools } from './tools/index.js';
import { registerAgentPrompts } from './agent/prompts.js';

async function main() {
	const server = new McpServer({
		name: 'blender-web',
		version: '0.1.0',
	});

	const sceneManager = new BlenderSceneManager();

	// Register all MCP tools
	registerTools(server, sceneManager);

	// Register agent prompts/resources
	registerAgentPrompts(server);

	// Start stdio transport
	const transport = new StdioServerTransport();
	await server.connect(transport);

	console.error('[blender-mcp] Server started on stdio');
}

main().catch(console.error);

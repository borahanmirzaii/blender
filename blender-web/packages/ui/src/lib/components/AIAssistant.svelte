<script lang="ts">
	import { sceneStore } from '$lib/stores/scene.svelte';

	interface Message {
		role: 'user' | 'assistant' | 'tool';
		content: string;
		toolCalls?: ToolCall[];
	}

	interface ToolCall {
		name: string;
		args: Record<string, any>;
		result?: string;
	}

	let messages = $state<Message[]>([
		{
			role: 'assistant',
			content: 'I\'m your Blender AI assistant. I can help you create objects, modify scenes, apply materials, and more. Try asking me to build something!'
		}
	]);
	let input = $state('');
	let isLoading = $state(false);
	let chatContainer: HTMLDivElement;

	// Available MCP tools that the AI can call
	const mcpTools = [
		{ name: 'create_object', description: 'Add a mesh primitive to the scene' },
		{ name: 'modify_mesh', description: 'Transform or modify an object' },
		{ name: 'set_material', description: 'Set material properties' },
		{ name: 'query_scene', description: 'Get information about the scene' },
		{ name: 'delete_object', description: 'Remove an object' },
		{ name: 'export_scene', description: 'Export the scene to JSON' },
	];

	async function sendMessage() {
		if (!input.trim() || isLoading) return;

		const userMessage = input.trim();
		input = '';
		messages = [...messages, { role: 'user', content: userMessage }];
		isLoading = true;

		// Simulate AI processing with MCP tool calls
		// In production, this calls the Claude API with MCP tools
		const response = await processWithMCP(userMessage);
		messages = [...messages, response];
		isLoading = false;

		// Scroll to bottom
		requestAnimationFrame(() => {
			if (chatContainer) {
				chatContainer.scrollTop = chatContainer.scrollHeight;
			}
		});
	}

	async function processWithMCP(userMessage: string): Promise<Message> {
		// Pattern matching for common requests
		// In production, this is handled by Claude API with tool_use
		const lower = userMessage.toLowerCase();

		if (lower.includes('add') || lower.includes('create')) {
			let type = 'cube';
			if (lower.includes('sphere')) type = 'sphere';

			const name = type.charAt(0).toUpperCase() + type.slice(1);
			sceneStore.addPrimitive(type, name);

			return {
				role: 'assistant',
				content: `Created a ${type} named "${name}". You can see it in the viewport and outliner.`,
				toolCalls: [{
					name: 'create_object',
					args: { type, name },
					result: `Object "${name}" created at origin`
				}]
			};
		}

		if (lower.includes('delete') || lower.includes('remove')) {
			if (sceneStore.activeObject) {
				const name = sceneStore.activeObject.name;
				sceneStore.deleteActive();
				return {
					role: 'assistant',
					content: `Deleted "${name}" from the scene.`,
					toolCalls: [{ name: 'delete_object', args: { name }, result: 'Deleted' }]
				};
			}
			return { role: 'assistant', content: 'No object selected. Select an object in the outliner first.' };
		}

		if (lower.includes('how many') || lower.includes('list') || lower.includes('scene')) {
			const names = sceneStore.objects.map(o => o.name).join(', ');
			return {
				role: 'assistant',
				content: `The scene has ${sceneStore.objects.length} objects: ${names}`,
				toolCalls: [{ name: 'query_scene', args: {}, result: names }]
			};
		}

		if (lower.includes('export')) {
			const json = sceneStore.exportJson();
			return {
				role: 'assistant',
				content: 'Scene exported. In production, this would save via WASI to your local filesystem or download as a file.',
				toolCalls: [{ name: 'export_scene', args: { format: 'json' }, result: `${json?.length ?? 0} bytes` }]
			};
		}

		return {
			role: 'assistant',
			content: `I understand you want to "${userMessage}". In production, this would call the Claude API with MCP tools for:\n\n${mcpTools.map(t => `- **${t.name}**: ${t.description}`).join('\n')}\n\nTry: "add a sphere", "delete selected", "list scene objects"`
		};
	}

	function onKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			sendMessage();
		}
	}
</script>

<div class="ai-assistant">
	<div class="ai-header">
		<span>AI Assistant</span>
		<span class="badge">MCP</span>
	</div>

	<div class="chat-messages" bind:this={chatContainer}>
		{#each messages as msg}
			<div class="message {msg.role}">
				<div class="message-content">
					{@html msg.content.replace(/\n/g, '<br>').replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')}
				</div>
				{#if msg.toolCalls}
					<div class="tool-calls">
						{#each msg.toolCalls as call}
							<div class="tool-call">
								<span class="tool-name">{call.name}</span>
								<span class="tool-result">{call.result}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/each}
		{#if isLoading}
			<div class="message assistant loading">
				<span class="dots">...</span>
			</div>
		{/if}
	</div>

	<div class="chat-input">
		<textarea
			bind:value={input}
			onkeydown={onKeyDown}
			placeholder="Ask the AI to modify your scene..."
			rows="2"
		></textarea>
		<button onclick={sendMessage} disabled={isLoading || !input.trim()}>
			Send
		</button>
	</div>
</div>

<style>
	.ai-assistant {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--bg-secondary);
	}

	.ai-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 10px;
		background: var(--bg-header);
		border-bottom: 1px solid var(--border);
		font-size: 11px;
		font-weight: 600;
	}

	.badge {
		background: var(--accent);
		color: white;
		padding: 1px 6px;
		border-radius: 3px;
		font-size: 9px;
		font-weight: 700;
	}

	.chat-messages {
		flex: 1;
		overflow-y: auto;
		padding: 8px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.message {
		padding: 8px 10px;
		border-radius: 6px;
		font-size: 12px;
		line-height: 1.5;
	}

	.message.user {
		background: var(--accent);
		color: white;
		align-self: flex-end;
		max-width: 85%;
	}

	.message.assistant {
		background: var(--bg-tertiary);
		align-self: flex-start;
		max-width: 95%;
	}

	.message.loading {
		opacity: 0.5;
	}

	.dots {
		animation: blink 1s infinite;
	}

	@keyframes blink {
		50% { opacity: 0.3; }
	}

	.tool-calls {
		margin-top: 6px;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.tool-call {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 10px;
		padding: 3px 6px;
		background: rgba(0, 0, 0, 0.2);
		border-radius: 3px;
		font-family: var(--font-mono);
	}

	.tool-name {
		color: var(--accent);
		font-weight: 600;
	}

	.tool-result {
		color: var(--success);
	}

	.chat-input {
		display: flex;
		gap: 6px;
		padding: 8px;
		border-top: 1px solid var(--border);
	}

	.chat-input textarea {
		flex: 1;
		resize: none;
		font-size: 12px;
		padding: 6px 8px;
		border-radius: 4px;
		font-family: var(--font-ui);
	}

	.chat-input button {
		padding: 6px 14px;
		background: var(--accent);
		border-color: var(--accent);
		color: white;
		font-size: 11px;
		align-self: flex-end;
	}

	.chat-input button:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>

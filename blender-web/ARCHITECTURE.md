# Blender Web: Rust/WASM + Svelte + AI Architecture

## Vision

Transform Blender's core capabilities into a modern, browser-native 3D platform
powered by Rust/WASM for performance, Svelte for the UI, WASI for local file
access, edge workers for heavy compute, and AI/MCP for intelligent assistance.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        BROWSER (Client)                            │
│                                                                    │
│  ┌──────────────┐  ┌──────────────────┐  ┌────────────────────┐   │
│  │  Svelte UI   │  │  WebGPU Renderer │  │  WASM Runtime      │   │
│  │              │◄─┤                  │◄─┤                    │   │
│  │  - Viewport  │  │  - Scene render  │  │  - blender-core    │   │
│  │  - Outliner  │  │  - Shader graph  │  │  - Mesh ops        │   │
│  │  - Props     │  │  - Post-process  │  │  - Scene graph     │   │
│  │  - Timeline  │  │                  │  │  - Modifiers       │   │
│  │  - Node Ed.  │  │                  │  │  - Physics (light) │   │
│  └──────┬───────┘  └──────────────────┘  └─────────┬──────────┘   │
│         │                                          │               │
│  ┌──────┴──────────────────────────────────────────┴──────────┐   │
│  │              WASI Runtime (wasmtime / browser polyfill)     │   │
│  │                                                            │   │
│  │  - File System Access API ←→ WASI fd_read/fd_write         │   │
│  │  - .blend file import/export                               │   │
│  │  - Texture/asset loading from local disk                   │   │
│  │  - Project directory watching                              │   │
│  └────────────────────────────┬───────────────────────────────┘   │
│                               │                                    │
└───────────────────────────────┼────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │    Network Layer       │
                    │  (WebSocket + HTTP/3)  │
                    └───────────┬───────────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          │                     │                     │
  ┌───────▼───────┐   ┌────────▼────────┐   ┌───────▼────────┐
  │  Edge Worker   │   │  Render Server  │   │  MCP / AI      │
  │  (Cloudflare/  │   │  (GPU Cloud)    │   │  Agent Server  │
  │   Deno Deploy) │   │                 │   │                │
  │                │   │  - Cycles WASM  │   │  - Tool calls  │
  │  - Auth/collab │   │  - Final render │   │  - Scene gen   │
  │  - Asset CDN   │   │  - Baking       │   │  - Assistance  │
  │  - Session mgmt│   │  - Simulation   │   │  - Code gen    │
  └────────────────┘   └─────────────────┘   └────────────────┘
```

## Layer Breakdown

### 1. Rust/WASM Core (`crates/`)

The heart of the system. Blender's core algorithms rewritten in Rust and
compiled to WebAssembly via `wasm-bindgen` and `wasm-pack`.

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| `blender-core` | Scene graph, mesh data, materials, transforms | `Scene`, `Mesh`, `Object`, `Material` |
| `blender-wasm` | wasm-bindgen API surface for browser | JS-facing bindings |
| `blender-wasi` | WASI-compatible file I/O, .blend parsing | `BlendFile`, `FileSystem` |
| `blender-gpu` | WebGPU abstraction, shader compilation | `Renderer`, `Pipeline`, `ShaderGraph` |

**Why Rust?**
- Zero-cost abstractions over Blender's C data structures
- `wasm32-unknown-unknown` target for browser, `wasm32-wasip1` for WASI
- Memory safety without GC pauses during 3D operations
- Shared code between browser and edge/server

### 2. Svelte Frontend (`packages/ui/`)

SvelteKit replaces Blender's custom GHOST window system with a reactive,
component-based UI that runs in any browser.

**Key components:**
- `Viewport3D.svelte` — WebGPU canvas with orbit/pan/zoom, gizmos
- `Outliner.svelte` — Scene hierarchy tree
- `Properties.svelte` — Context-sensitive property editor
- `Timeline.svelte` — Animation keyframe editor
- `NodeEditor.svelte` — Visual node graph (shader/geometry/compositor)
- `AIAssistant.svelte` — AI chat panel with MCP tool results

**Why Svelte?**
- Compiled away at build time → minimal runtime overhead
- Reactive stores map naturally to Blender's depsgraph pattern
- SvelteKit provides SSR, routing, and edge deployment

### 3. WASI + Local File Access

Two-pronged approach for file system access:

**Browser (File System Access API):**
```
User grants directory access → OPFS or native FS handle
→ Polyfilled as WASI fd_* calls → Rust reads/writes .blend files
```

**WASI Runtime (Server/Edge):**
```
wasmtime/wasmer runs blender-wasi.wasm
→ Native fd_read/fd_write → Real filesystem
→ Same Rust code, different host
```

This means the same Rust `.blend` parser works in both browser and server.

### 4. Edge / Server Interaction

| Tier | Runtime | Responsibilities |
|------|---------|-----------------|
| **Edge** | Cloudflare Workers / Deno Deploy | Auth, session state, asset proxy, collaboration |
| **GPU Server** | Dedicated GPU instances | Final renders (Cycles), physics sim, baking |
| **AI Server** | Claude API / local LLM | MCP tool execution, scene generation, code assist |

**Communication protocol:**
- WebSocket for real-time collaboration and viewport sync
- HTTP/3 for asset uploads/downloads
- Server-Sent Events for render progress and AI streaming

### 5. AI / Agent / MCP Integration

The AI layer uses the **Model Context Protocol (MCP)** to expose Blender
operations as tools that Claude or other LLMs can invoke.

**MCP Tools:**
| Tool | Description |
|------|-------------|
| `create_object` | Add mesh/light/camera to scene |
| `modify_mesh` | Apply transforms, modifiers, sculpt ops |
| `set_material` | Create/assign PBR materials |
| `render_preview` | Trigger viewport render, return image |
| `export_scene` | Export to glTF/USD/FBX |
| `query_scene` | Inspect scene graph, find objects |
| `run_geometry_nodes` | Execute a geometry node graph |
| `animate` | Set keyframes, create animation curves |

**Agent Workflows:**
```
User: "Create a low-poly forest scene with a cabin"
  → Agent plans: terrain → trees → cabin → materials → lighting
  → Each step calls MCP tools
  → User sees real-time viewport updates
  → Agent iterates based on render previews
```

## Build & Run

```bash
# Build WASM crates
cd blender-web
cargo build --target wasm32-unknown-unknown -p blender-wasm
wasm-pack build crates/blender-wasm --target web

# Build WASI module
cargo build --target wasm32-wasip1 -p blender-wasi

# Run Svelte dev server
cd packages/ui
npm install && npm run dev

# Run MCP server
cd packages/mcp-server
npm install && npm run dev
```

## Migration Strategy

This is NOT a full rewrite of Blender. It's a progressive extraction:

1. **Phase 1** — Core data structures in Rust (Scene, Mesh, Object, Material)
2. **Phase 2** — WebGPU viewport with basic editing
3. **Phase 3** — .blend file I/O via WASI
4. **Phase 4** — Edge worker for collaboration
5. **Phase 5** — AI/MCP integration for intelligent tools
6. **Phase 6** — Server-side Cycles rendering
7. **Phase 7** — Full feature parity roadmap

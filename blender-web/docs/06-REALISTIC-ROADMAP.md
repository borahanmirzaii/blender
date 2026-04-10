# Blender Web — Realistic Roadmap: CAD-First, Then Beyond

## Philosophy: Start Simple, Ship Fast, Grow Organically

> "A working demo beats a thousand architecture docs."
> — Lesson from CADmium's failure

### What Killed CADmium
CADmium (Rust+WASM+Svelte+Truck B-rep) was discontinued because:
1. B-rep kernel (Truck) was immature for production use
2. Tried to build full parametric CAD before having basic editing working
3. No incremental delivery — users had nothing to use until "done"

### What Made Figma Succeed
1. Started as a **simple vector editor** — not a full design tool
2. C++ core compiled to WASM — fast from day one
3. React UI around the canvas — familiar web development
4. Added features based on user demand, not a master plan

---

## Our Path: 5 Stages from CAD to Beyond

```
Stage 1: SCENE VIEWER          "Look at 3D things in a browser"
Stage 2: SIMPLE CAD            "Move, scale, arrange objects"  
Stage 3: MESH EDITING          "Edit geometry like Blender"
Stage 4: SMART TOOLS           "AI helps you build scenes"
Stage 5: FULL PLATFORM         "Web-native Blender alternative"
```

---

### Stage 1: Scene Viewer (Week 1-4)
**What users get**: Drop a .gltf file, see it rendered in browser.

```
Rust (blender-core):
  - Scene, Object, Mesh, Material structs
  - glTF import (via gltf crate)
  - Camera math (view/projection matrices)

Rust (blender-gpu):  
  - wgpu initialization (native + WASM)
  - PBR shader (WGSL)
  - Multi-object draw loop

WASM bridge:
  - SceneHandle: load_gltf(bytes), object_count(), etc.
  - RendererHandle: init(canvas), render_frame()

Svelte:
  - Canvas + WebGPU context
  - Orbit camera (mouse drag)
  - Object list sidebar
  - File drop zone
```

**Why start here**: Zero mesh editing needed. Import-only. Proves the
full pipeline: Rust → WASM → WebGPU → pixels on screen.

**Comparable to**: Three.js editor viewer, Sketchfab viewer.

---

### Stage 2: Simple CAD (Week 5-12)
**What users get**: Add primitives, move/rotate/scale, save scenes.

```
New capabilities:
  + Add cube/sphere/cylinder/plane
  + Transform gizmos (translate, rotate, scale)
  + Object picking (click to select)
  + Delete objects
  + Undo/redo (bincode snapshots)
  + Material editor (color, metallic, roughness)
  + Save/load scenes (JSON, File System Access API)
  + Multiple viewports? (optional)
  + Grid snapping
```

**Why this matters**: Users can assemble scenes. Not modeling, but scene
composition. Think architectural visualization layout, product staging.

**Comparable to**: Spline.design, basic Unity scene editor.

---

### Stage 3: Mesh Editing (Week 13-24)
**What users get**: Edit vertices/edges/faces like Blender's edit mode.

```
New capabilities:
  + EditMesh (SlotMap half-edge)
  + Enter/exit edit mode
  + Vertex/edge/face selection
  + Extrude, loop cut, bevel
  + Subdivision surface modifier
  + Mirror modifier
  + Boolean modifier (via csgrs)
  + Basic UV editing
```

**Why this is hard**: Half-edge mesh in Rust is a known pain point.
SlotMap arena allocation is the best approach but still more complex
than C pointers. Budget extra time.

**Comparable to**: Blender edit mode (subset), Plasticity (simple version).

---

### Stage 4: Smart Tools (Week 25-36)
**What users get**: AI-assisted scene building and editing.

```
New capabilities:
  + MCP server with scene tools
  + AI Assistant panel (Claude integration)
  + "Create a forest scene" → AI builds it step-by-step
  + Material suggestions from text description
  + Geometry nodes (basic procedural modeling)
  + Animation (keyframes, timeline, playback)
  + Server-side rendering (Cloudflare Worker → GPU server)
  + Real-time collaboration (WebSocket)
```

**Why AI matters here**: This is where we differentiate from "just another
3D editor." AI-native tools that don't exist in desktop Blender.

**Comparable to**: Blender + BlenderMCP, but natively integrated.

---

### Stage 5: Full Platform (Ongoing)
**What users get**: A complete web-native DCC tool.

```
Future capabilities:
  + Sculpting (multires)
  + Texture painting
  + Full shader node editor
  + Particle systems
  + Physics simulation
  + Video sequence editor
  + Scripting (Lua or JS, not Python)
  + Plugin/extension system
  + Desktop app via Tauri
  + Mobile support (touch + WebGPU)
```

---

## What We Should Build RIGHT NOW

Not Stage 5. Not even Stage 2. We need **Stage 1** working end-to-end:

1. Rust crate that represents a scene with meshes
2. Compiled to WASM with wasm-pack
3. Svelte page that loads the WASM
4. Canvas that renders the scene (even with Canvas2D initially)
5. Mouse-driven camera orbit

**That's it.** Once this works, everything else is incremental.

---

## Technology Stack (Validated by Research)

| Layer | Technology | Status | Risk |
|-------|-----------|--------|------|
| Language | Rust 2024 edition | Stable | Low |
| Math | glam v0.32 | Stable, WASM-ready | Low |
| GPU | wgpu v29 | Stable, WASM+WebGPU | Low |
| WASM target | wasm32-unknown-unknown | Tier 1 in rustc | Low |
| WASM bridge | wasm-bindgen 0.2 | Stable | Low |
| WASM build | wasm-pack | Stable | Low |
| UI framework | Svelte 5 + SvelteKit | Stable | Low |
| Serialization | serde + bincode | Stable | Low |
| Arena allocation | slotmap | Stable, WASM-ready | Low |
| Graph structure | petgraph | Stable, WASM-ready | Low |
| 3D import | gltf crate | Stable, WASM-ready | Low |
| Mesh editing | Custom (SlotMap-based) | Must build | **Medium** |
| Subdivision | Custom | Must build | **Medium** |
| CSG/Booleans | csgrs (float-based) | Exists, not exact | **Medium** |
| AI integration | Claude API + MCP | Stable | Low |
| File access | File System Access API | Chrome/Edge only | **Medium** |
| Desktop | Tauri | Stable | Low |

All "Low risk" items are production-proven and WASM-compatible.
"Medium risk" items need custom implementation or have browser gaps.

---

## Comparison with ChatGPT's Assessment

| ChatGPT Said | Our Assessment | Agreement |
|-------------|----------------|-----------|
| Full port = extremely difficult | Confirmed: 2.96M LOC, 13 circular deps | Yes |
| Rebuild > Port | Confirmed: new architecture, not translation | Yes |
| Start with scene graph + mesh editing + renderer | Yes, but even simpler: viewer first | Mostly |
| "Next-gen web-native Blender alternative" | Exactly right | Yes |
| GPU access tricky | WebGPU now shipped everywhere (desktop) | Was true, improving |
| WASM still slower | ~90-95% native, WASM 3.0 threads+SIMD | Narrowing fast |
| Don't reproduce everything at once | 100% agreed — CADmium learned this the hard way | Yes |

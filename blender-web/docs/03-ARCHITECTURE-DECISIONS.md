# Blender Web — Architecture Decision Records (ADRs)

---

## ADR-001: Arena Allocation for Mesh Data (SlotMap)

**Status**: Accepted

**Context**: Blender's BMesh uses raw C pointers for half-edge topology
(disk cycles, radial cycles). Rust's ownership model fundamentally conflicts
with graph data structures that have shared, cyclical references.

**Options considered**:
1. `Rc<RefCell<T>>` — runtime-checked shared ownership
2. Raw pointers with `unsafe` — C-like approach
3. ECS (bevy_ecs) — entity-component decomposition
4. **Arena allocation (SlotMap)** — typed indices into dense storage

**Decision**: Use `slotmap::SlotMap` with typed keys.

**Rationale**:
- Generational indices detect use-after-free at runtime (unlike raw pointers)
- O(1) insert/remove, dense iteration (cache-friendly)
- No `RefCell` runtime panics, no lifetime gymnastics
- `Rc<RefCell<T>>` adds 16+ bytes overhead per element (40M+ allocations
  for a 10M vertex mesh = 640MB overhead)
- ECS is wrong abstraction — mesh topology needs adjacency, not component queries

**Consequences**: All mesh topology code uses indices instead of references.
Slightly less ergonomic than pointers but dramatically safer.

---

## ADR-002: wgpu as Sole GPU Backend

**Status**: Accepted

**Context**: Blender maintains its own GPU abstraction (`GPU_*` API) wrapping
OpenGL, Vulkan, and Metal — 107K LOC. We need cross-platform GPU support
including WebGPU for browsers.

**Decision**: Use `wgpu` directly. No custom GPU abstraction.

**Rationale**:
- wgpu already abstracts Vulkan, Metal, D3D12, OpenGL, **and WebGPU**
- Same Rust code compiles to native AND WASM without changes
- Production-proven (Firefox, Deno, Bevy — millions of users)
- Maintained by gfx-rs team with Mozilla backing
- Shader translation via Naga (WGSL ↔ SPIR-V/MSL/HLSL)
- Eliminates 107K LOC of abstraction code

**Consequences**: We depend on wgpu's API surface and release cycle.
GPU features limited to what wgpu exposes (which tracks WebGPU spec closely).

---

## ADR-003: Svelte 5 for UI Layer

**Status**: Accepted

**Context**: Blender's UI is a custom immediate-mode system (GHOST windowing
+ custom widget drawing) — 792K LOC in editors alone. We need a web UI
framework for browser deployment.

**Options considered**:
1. React — largest ecosystem, but heavy runtime, complex state management
2. Vue — good reactive model, but larger bundle than Svelte
3. **Svelte 5** — compiled away, small runtime, reactive runes
4. Solid.js — fine-grained reactivity, smaller ecosystem
5. Custom WASM UI (like Blender's own) — maximum control, massive effort

**Decision**: Svelte 5 with SvelteKit.

**Rationale**:
- Compiled approach → minimal runtime overhead (critical for 3D app)
- Runes system (`$state`, `$derived`, `$effect`) maps naturally to
  Blender's property update → depsgraph → redraw pattern
- SvelteKit provides routing, SSR, edge deployment
- CADmium used this exact stack (SvelteKit + Rust WASM)
- Svelte + Tauri = native desktop path via Rust backend
- `.svelte.ts` files allow shared reactive state outside components

**Consequences**: Smaller ecosystem than React. Some Svelte 5 patterns
still maturing. Team must learn runes system.

---

## ADR-004: Hybrid Architecture (Rust WASM Core + TypeScript UI)

**Status**: Accepted

**Context**: Full 3D app in browser. Need to decide what runs in WASM
vs JavaScript.

**Decision**: Split by hot/cold path:
```
WASM (Rust):                    JS/TS (Svelte):
├─ Scene graph                  ├─ UI layout/panels
├─ Mesh operations              ├─ Property editors
├─ Modifier evaluation          ├─ Menu/toolbar
├─ Depsgraph                    ├─ File dialogs
├─ Node graph evaluation        ├─ Animation timeline (UI)
├─ Rendering pipeline           ├─ Node editor (UI)
├─ Math/geometry                ├─ AI assistant
├─ File parsing                 └─ DOM event handling
└─ Collision/picking
```

**Rationale**:
- Figma uses this exact split (C++ WASM core + React UI)
- Minimizes JS↔WASM boundary crossing (expensive: ~100ns per call)
- UI events stay in JS; only 3D interaction crosses to WASM
- Svelte can reactively update from WASM state changes
- Each side uses its strengths (Rust: performance, JS: DOM)

**Consequences**: Must carefully design the WASM↔JS bridge API.
Shared memory via `SharedArrayBuffer` for large data (vertex buffers).
Small state via `wasm-bindgen` typed API.

---

## ADR-005: Proc Macro Reflection Instead of Code Generation

**Status**: Accepted

**Context**: Blender's RNA system uses hand-written C code to register
every property of every type (~196K LOC in makesrna). This provides
runtime reflection for UI, Python, animation, and serialization.

**Decision**: Use Rust proc macros to derive reflection at compile time.

**Rationale**:
- Proc macros generate code at compile time (zero runtime cost)
- `#[derive(BlenderData)]` replaces 196K LOC of hand-written registration
- `inventory` crate enables global type registry (like RNA's global `BlenderRNA`)
- Compile-time type safety vs RNA's runtime type checking
- Property metadata embedded in generated code, not separate registration files

**Consequences**: Must build and maintain the `blender-derive` proc macro
crate. Compile times increase proportional to macro complexity.
Debugging proc macro output requires `cargo expand`.

---

## ADR-006: Petgraph for Dependency Graph

**Status**: Accepted

**Context**: Blender's depsgraph is a custom DAG implementation (~25K LOC)
with topological sorting, lazy evaluation, and parallel scheduling.

**Decision**: Use `petgraph::StableDiGraph` as the foundation.

**Rationale**:
- StableDiGraph maintains stable node/edge indices across removal
- Built-in topological sort (`petgraph::algo::toposort`)
- Well-tested, widely used in Rust ecosystem
- Add custom dirty tracking via `BitVec` overlay
- Add COW evaluation store on top
- 25K LOC of custom graph code → <2K LOC using petgraph

**Consequences**: petgraph's generic API adds some boilerplate. Performance
may require custom iteration for hot paths (but petgraph is already fast).

---

## ADR-007: WASI Shim for Browser File Access

**Status**: Accepted

**Context**: Users need to read/write files (projects, textures, exports)
from the browser. WASI provides filesystem abstractions, but browsers
don't natively support WASI.

**Decision**: Dual-target file I/O:
- Browser: File System Access API → WASI shim (`browser_wasi_shim`)
- Native: Standard WASI (`wasm32-wasip1`) or native Rust `std::fs`

**Rationale**:
- Same Rust file I/O code works in both browser and server/native
- File System Access API is shipped in Chrome/Edge, polyfillable elsewhere
- OPFS (Origin Private File System) for auto-save / scratch files
- User explicitly grants directory access (security model)

**Consequences**: File System Access API not available in Firefox/Safari
for directory access (file-by-file only). Must provide fallback
(download/upload) for unsupported browsers.

---

## ADR-008: MCP for AI Integration

**Status**: Accepted

**Context**: AI-assisted 3D modeling is a key differentiator. Need to
expose scene operations to LLMs.

**Decision**: Use Model Context Protocol (MCP) with stdio transport.

**Rationale**:
- MCP is the emerging standard for AI↔tool integration
- Claude, GPT, and other models support MCP tool calling
- Stdio transport works locally; HTTP/SSE for remote
- Tools map 1:1 to operators (create_object = "mesh.primitive_cube_add")
- Agent prompts provide domain expertise (scene builder, material designer)
- Closed feedback loop possible: render → screenshot → Claude vision → next tool call

**Consequences**: MCP server runs as separate process (not in browser WASM).
Browser AI features proxy through edge worker to MCP server.

---

## ADR-009: No ECS for Core Data Model

**Status**: Accepted

**Context**: ECS (Entity-Component-System) is popular in Rust game engines.
Should we use bevy_ecs for Blender's data model?

**Decision**: Do NOT use ECS for the core data model. Use traditional
object model with traits.

**Rationale**:
- Blender's data model is **object-oriented with rich relationships**
  (parent/child, constraints, drivers, modifiers, shape keys)
- ECS optimizes for **cache-coherent iteration over homogeneous components**
  — wrong optimization target for a DCC tool
- An Object in Blender has 50+ fields with complex interdependencies
  — decomposing into components loses semantic coherence
- DNA/RNA reflection system requires inspectable struct fields
  — ECS archetypes are opaque to reflection
- Depsgraph already provides the evaluation scheduling that ECS systems offer

**Exception**: ECS MAY be used for particle systems and instancing where
cache-coherent iteration over millions of identical elements matters.

**Consequences**: More traditional Rust code structure. Less "Rusty" than
ECS advocates would prefer. But correctly models the domain.

---

## ADR-010: Incremental Migration Strategy

**Status**: Accepted

**Context**: Blender is 2.96M LOC. A full rewrite is infeasible.
We need a strategy that delivers value incrementally.

**Decision**: Phase-based extraction with running software at each phase.

**Phases**:
1. **Viewer** — Load scene data, render viewport (read-only)
2. **Editor** — Add object manipulation, transform, basic editing
3. **Modeler** — BMesh editing, modifiers, geometry nodes
4. **Animator** — Keyframes, playback, constraints
5. **Renderer** — EEVEE-equivalent real-time + server-side Cycles
6. **Full DCC** — Sculpting, UV, painting, full node editors, scripting

**Rationale**:
- Each phase delivers a usable product
- Viewer alone is valuable (web .blend viewer)
- Editor enables basic scene assembly
- Later phases add depth based on demand
- Features can be prioritized by user research

**Consequences**: Some architectural decisions must be forward-looking
(design for modifiers even if Phase 1 doesn't use them).
Risk of rework if early assumptions prove wrong.

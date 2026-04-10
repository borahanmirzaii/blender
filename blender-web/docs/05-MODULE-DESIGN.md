# Blender Web — Module Design & Dependency Architecture

## The Core Problem: Breaking Blender's Circular Dependencies

Blender's `blenkernel` has bidirectional dependencies with 13+ modules.
This evolved over 30 years and is acknowledged by Blender developers as
technical debt. Our Rust architecture MUST NOT replicate this.

### Strategy: Dependency Inversion via Traits

Instead of Module A depending on Module B and vice versa, both depend
on shared **trait definitions** in `blender-core`.

```
BLENDER (circular):                BLENDER-WEB (acyclic):

  blenkernel ◄──► depsgraph          blender-core (traits)
  blenkernel ◄──► modifiers              ▲         ▲
  blenkernel ◄──► nodes             blender-mesh  blender-depsgraph
  blenkernel ◄──► gpu               (implements   (implements
  blenkernel ◄──► draw               MeshOps)      EvalGraph)
```

---

## Crate Architecture

### Layer 0: Foundation

#### `blender-derive` (proc macro crate)
```
Purpose: #[derive(BlenderData)] proc macro
Depends on: syn, quote, proc-macro2
Output:   impl BlenderData for T
          impl Serialize/Deserialize for T
          PropertyDef metadata
          TypeInfo registration via inventory
```

#### `blender-core` (trait definitions + common types)
```
Purpose: ALL shared abstractions. No implementation logic.
Depends on: glam, serde, bitflags, slotmap (re-exported)

Modules:
  core::id         — DataBlockId, trait DataBlock, ID flags
  core::scene      — Scene struct, ViewLayer, World
  core::object     — Object struct, ObjectData enum, ObjectRuntime
  core::transform  — Transform, parent resolution
  core::mesh       — Mesh struct (evaluated), Vertex, attribute types
  core::material   — Material (PBR), shader slot
  core::camera     — Camera, projection types
  core::light      — Light types (point, sun, spot, area)
  core::modifier   — trait Modifier, ModifierType enum
  core::node       — trait NodeType, SocketValue, NodeDeclaration
  core::depsgraph  — trait DepsgraphEval, EvalNode, Relation
  core::operator   — trait Operator, OperatorResult, UndoType
  core::geometry   — GeometrySet, GeometryComponent enum
  core::attribute  — AttributeDomain, CustomDataType, LayerStorage
  core::event      — ViewportEvent, KeyCode, MouseButton
  core::context    — Context struct (references to active state)
  core::property   — PropertyDef, PropertyValue, trait BlenderData
  core::anim       — Keyframe, FCurve, AnimData
```

**Critical rule**: `blender-core` contains ONLY:
- Struct definitions (data)
- Trait definitions (interfaces)
- Enum definitions (type unions)
- No business logic, no algorithms

### Layer 1: Data Structures

#### `blender-mesh`
```
Purpose: Half-edge mesh data structure + editing operators
Depends on: blender-core, slotmap, glam

Modules:
  mesh::edit       — EditMesh (SlotMap half-edge)
  mesh::topology   — Disk/radial cycle operations
  mesh::ops        — Extrude, loop cut, bevel, inset, merge, fill
  mesh::convert    — Mesh ↔ EditMesh conversion
  mesh::normals    — Normal calculation
  mesh::primitives — Cube, sphere, plane, torus, cylinder generators
  mesh::subdivide  — Catmull-Clark, simple subdivision
  mesh::boolean    — CSG operations (union, difference, intersect)
  mesh::decimate   — Mesh simplification
  mesh::query      — BVH tree, ray-cast, closest point
```

#### `blender-animation`
```
Purpose: Keyframe and animation data
Depends on: blender-core, glam

Modules:
  anim::fcurve     — FCurve evaluation (Bezier, linear, constant)
  anim::keyframe   — Keyframe insertion, deletion, modification
  anim::driver     — Expression-driven values
  anim::constraint — Object/bone constraints (TrackTo, CopyLocation, etc.)
  anim::action     — Action clips, NLA strips
```

### Layer 2: Evaluation

#### `blender-depsgraph`
```
Purpose: Dependency graph construction and evaluation
Depends on: blender-core, petgraph, bitvec, rayon

Modules:
  depsgraph::graph    — StableDiGraph wrapper with typed nodes
  depsgraph::builder  — Construct graph from Scene
  depsgraph::eval     — Topological evaluation with parallelism
  depsgraph::cow      — Copy-on-Write store for evaluated data
  depsgraph::tags     — Dirty flag propagation
  depsgraph::schedule — Parallel scheduling via rayon
```

#### `blender-modifiers`
```
Purpose: Modifier implementations
Depends on: blender-core, blender-mesh

Modules:
  modifiers::subdivision   — Catmull-Clark
  modifiers::mirror        — Mirror across axis
  modifiers::array         — Linear/radial array
  modifiers::boolean       — CSG boolean
  modifiers::solidify      — Shell/rim generation
  modifiers::bevel         — Edge/vertex bevel
  modifiers::decimate      — Mesh reduction
  modifiers::smooth        — Laplacian smoothing
  modifiers::displace      — Texture-based displacement
  modifiers::evaluate      — Stack evaluation pipeline
```

#### `blender-nodes`
```
Purpose: Node graph system (geometry, shader, compositor)
Depends on: blender-core, blender-mesh, petgraph

Modules:
  nodes::graph     — NodeGraph, Node, Socket, Link
  nodes::eval      — Topological evaluation
  nodes::field     — Field system (vectorized per-element)
  nodes::geometry  — Geometry node implementations
  nodes::shader    — Shader node implementations
  nodes::math      — Math/vector/color node library
  nodes::register  — Node type registry
```

### Layer 3: Rendering

#### `blender-gpu`
```
Purpose: wgpu-based rendering abstractions
Depends on: blender-core, wgpu, bytemuck, glam

Modules:
  gpu::context    — Device, queue, surface management
  gpu::shader     — WGSL shader loading, ShaderModule management
  gpu::batch      — Vertex/index buffer batching
  gpu::uniform    — Uniform buffer management, bind groups
  gpu::texture    — Texture loading, sampling, format conversion
  gpu::framebuffer — Render target management
  gpu::pipeline   — Render/compute pipeline creation
  gpu::camera     — OrbitCamera, viewport matrices
```

#### `blender-draw`
```
Purpose: Viewport rendering engines
Depends on: blender-core, blender-gpu, blender-mesh

Modules:
  draw::engine     — trait DrawEngine, engine registry
  draw::workbench  — Solid/wireframe viewport (fast)
  draw::eevee      — PBR real-time renderer
  draw::overlay    — Gizmos, selection highlights, grid, axes
  draw::picking    — GPU-based object/element picking
```

### Layer 4: I/O

#### `blender-io`
```
Purpose: File format reading/writing
Depends on: blender-core, blender-mesh

Modules:
  io::bweb       — Native .bweb format (bincode-based)
  io::blend      — .blend file reader (incremental)
  io::gltf       — glTF 2.0 import/export
  io::obj        — OBJ/MTL import/export
  io::stl        — STL import/export
  io::usd        — USD import/export (future)
  io::version    — File version migration system
```

### Layer 5: WASM Bridge

#### `blender-wasm`
```
Purpose: wasm-bindgen API surface for browser
Depends on: ALL crates above
Crate type: cdylib

Modules:
  wasm::scene     — SceneHandle: create, load, save, query
  wasm::viewport  — RendererHandle: init, resize, render_frame
  wasm::edit      — EditHandle: enter/exit edit mode, mesh ops
  wasm::operator  — Execute operators from JS
  wasm::event     — Process viewport events
  wasm::export    — Export scene data for Svelte stores
```

### Layer 6: Frontend & Services

#### `packages/ui` (SvelteKit)
```
Depends on: blender-wasm (via WASM module)

src/lib/
  components/    — Svelte components matching Blender's editor types
    Viewport3D.svelte
    Outliner.svelte
    Properties.svelte
    Timeline.svelte
    NodeEditor.svelte
    AIAssistant.svelte
    Toolbar.svelte
    ModifierStack.svelte

  stores/        — Reactive state (Svelte 5 runes)
    scene.svelte.ts    — Scene state bridged from WASM
    selection.svelte.ts
    preferences.svelte.ts
    undo.svelte.ts

  wasm/          — WASM loading and bridge
    loader.ts
    bridge.ts    — Typed wrapper around wasm-bindgen API

  webgpu/        — WebGPU integration
    renderer.ts
    shaders.ts
```

#### `packages/edge-worker` (Cloudflare Workers)
```
Endpoints:
  /api/scene/:id     — CRUD for scene storage (KV)
  /api/assets/*      — CDN for textures/models (R2)
  /api/render        — Dispatch render jobs (Queue)
  /api/collab/:room  — WebSocket collaboration (Durable Objects)
  /api/ai            — Proxy to Claude API with MCP tools
```

#### `packages/mcp-server` (Model Context Protocol)
```
Tools:
  create_object    — Add mesh/light/camera
  modify_object    — Set transform
  set_material     — PBR material properties
  delete_object    — Remove from scene
  query_scene      — Find/list objects
  export_scene     — Serialize to JSON
  animate          — Set keyframes
  duplicate_object — Clone with offset
  edit_mesh        — BMesh operations (future)
  apply_modifier   — Add/configure modifiers (future)

Prompts:
  scene-builder    — Build scene from text description
  material-designer — Design PBR materials
  scene-analyzer   — Analyze and suggest improvements
  animator         — Create animations from description
```

---

## Data Flow Architecture

```
                    ┌──────────────────────────────────┐
                    │         Svelte UI Layer           │
                    │  (reactive stores + components)   │
                    └───────────────┬──────────────────┘
                                    │ wasm-bindgen API
                    ┌───────────────▼──────────────────┐
                    │         blender-wasm              │
                    │  (JS↔Rust bridge, thin facade)    │
                    └───────────────┬──────────────────┘
                                    │
          ┌─────────────────────────┼─────────────────────────┐
          │                         │                         │
┌─────────▼─────────┐   ┌──────────▼──────────┐   ┌─────────▼─────────┐
│  Operator System   │   │   Render Pipeline    │   │    File I/O       │
│  (command + undo)  │   │   (wgpu + draw)      │   │  (.bweb, .gltf)   │
└─────────┬─────────┘   └──────────┬──────────┘   └───────────────────┘
          │                         │
          │   ┌─────────────────────┘
          │   │
┌─────────▼───▼─────┐
│    Depsgraph       │
│ (evaluation DAG)   │
└─────────┬─────────┘
          │ evaluates
┌─────────▼─────────────────────────────────────────┐
│                   Scene Data                       │
│  ┌─────────┐ ┌──────────┐ ┌───────┐ ┌──────────┐ │
│  │ Objects │ │  Meshes  │ │ Mats  │ │ Modifiers│ │
│  └────┬────┘ └────┬─────┘ └───────┘ └──────────┘ │
│       │           │                                │
│  ┌────▼────┐ ┌────▼─────┐ ┌──────────┐           │
│  │ Animdata│ │ EditMesh │ │ NodeGraph│           │
│  └─────────┘ └──────────┘ └──────────┘           │
└───────────────────────────────────────────────────┘
```

---

## Compile Targets

| Target | Purpose | Build Command |
|--------|---------|---------------|
| `wasm32-unknown-unknown` | Browser (WebGPU) | `wasm-pack build --target web` |
| `wasm32-wasip1` | Server WASI / file I/O | `cargo build --target wasm32-wasip1` |
| Native (debug) | Development / testing | `cargo build` |
| Native (release) | Desktop app via Tauri | `cargo build --release` |

---

## Testing Strategy

| Level | Tool | Coverage |
|-------|------|----------|
| Unit tests | `cargo test` | Core data types, mesh ops, math |
| Integration tests | `cargo test` | Depsgraph eval, modifier chains |
| WASM tests | `wasm-pack test --headless` | Browser API surface |
| Visual regression | Screenshot comparison | Viewport rendering |
| Performance | `criterion` benchmarks | Mesh ops, render frame time |
| E2E | Playwright | Full UI interaction flows |

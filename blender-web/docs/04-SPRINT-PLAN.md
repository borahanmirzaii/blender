# Blender Web — Sprint Plan & Milestone Tracking

## Project Phases Overview

```
Phase 1: VIEWER        ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░  MVP
Phase 2: EDITOR        ░░░░░░░░████████░░░░░░░░░░░░░░░░░░░░░░
Phase 3: MODELER       ░░░░░░░░░░░░░░░░████████░░░░░░░░░░░░░░
Phase 4: ANIMATOR      ░░░░░░░░░░░░░░░░░░░░░░░░████████░░░░░░
Phase 5: RENDERER      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██████
Phase 6: FULL DCC      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (ongoing)
```

---

## Phase 1: VIEWER (MVP — Web .blend/.gltf Viewer)

**Goal**: Load a scene, render it in the browser, navigate the viewport.
**Deliverable**: Embeddable web viewer for 3D scenes.

### Sprint 1.1 — Foundation Crates
**Focus**: Core data types, build system, WASM compilation

| Task | Crate | Status | Notes |
|------|-------|--------|-------|
| Rust workspace setup with WASM targets | workspace | [ ] | Cargo.toml, wasm-pack config |
| `blender-derive` proc macro (`#[derive(BlenderData)]`) | blender-derive | [ ] | PropertyDef, TypeInfo generation |
| Core ID/DataBlock system | blender-core | [ ] | `trait DataBlock`, `DataBlockId`, `Main` |
| Vec3/Quat/Mat4 math (via glam) | blender-core | [ ] | Re-export glam, add convenience methods |
| Scene, Object, Transform structs | blender-core | [ ] | With `#[derive(BlenderData)]` |
| Mesh struct (evaluated, GPU-ready) | blender-core | [ ] | Vertex/Index buffers, attribute layers |
| Material struct (PBR metallic-roughness) | blender-core | [ ] | Maps to glTF model |
| Camera, Light structs | blender-core | [ ] | Perspective/Ortho, Point/Sun/Spot/Area |
| JSON serialization for all types | blender-core | [ ] | `serde_json` |
| Unit tests for all core types | blender-core | [ ] | Round-trip serialization, math correctness |

### Sprint 1.2 — GPU Renderer
**Focus**: WebGPU viewport rendering via wgpu

| Task | Crate | Status | Notes |
|------|-------|--------|-------|
| wgpu device/surface initialization | blender-gpu | [ ] | Works native + WASM |
| WGSL PBR shader (vertex + fragment) | blender-gpu | [ ] | Metallic-roughness, Reinhard tonemap |
| Orbit camera (view/projection matrices) | blender-gpu | [ ] | Orbit, pan, zoom matching Blender |
| Vertex buffer upload from Mesh | blender-gpu | [ ] | VertBuf + IndexBuf management |
| Uniform buffer management | blender-gpu | [ ] | Per-object, per-material, per-camera |
| Depth buffer + back-face culling | blender-gpu | [ ] | Standard pipeline state |
| Grid floor rendering | blender-gpu | [ ] | Infinite grid shader |
| Multi-object rendering loop | blender-gpu | [ ] | Draw all objects in scene |
| Viewport resize handling | blender-gpu | [ ] | Canvas resize observer |
| wasm-bindgen API for renderer | blender-wasm | [ ] | `init_renderer(canvas)`, `render_frame()` |

### Sprint 1.3 — Svelte UI Shell
**Focus**: Basic application layout with viewport integration

| Task | Package | Status | Notes |
|------|---------|--------|-------|
| SvelteKit project setup | ui | [ ] | Svelte 5, TypeScript, Vite |
| WASM loader module | ui | [ ] | Load .wasm, init, fallback |
| Blender-inspired dark theme CSS | ui | [ ] | CSS custom properties |
| Main layout (header, viewport, sidebar) | ui | [ ] | Flexbox-based, resizable |
| Viewport3D component (canvas + wgpu) | ui | [ ] | Mouse events → WASM |
| Outliner component (scene tree) | ui | [ ] | Hierarchical object list |
| Properties panel (read-only) | ui | [ ] | Object/Material properties |
| Scene reactive store (`$state`) | ui | [ ] | Bridge WASM scene → Svelte state |
| glTF import (via `gltf` crate) | blender-io | [ ] | Load .gltf/.glb files |
| Drag-and-drop file loading | ui | [ ] | Drop .gltf onto viewport |

### Sprint 1.4 — Polish & Deploy
| Task | Status | Notes |
|------|--------|-------|
| WASM bundle optimization (wasm-opt, tree-shaking) | [ ] | Target <5MB gzipped |
| Loading screen with progress | [ ] | WASM download progress |
| Error handling + WebGPU fallback messages | [ ] | Detect unsupported browsers |
| Embed as `<iframe>` / web component | [ ] | For blog posts, docs |
| Deploy to Cloudflare Pages / Vercel | [ ] | Static site |
| README with usage instructions | [ ] | |
| Performance benchmarking | [ ] | FPS, load time, memory |

**Phase 1 Exit Criteria**:
- [ ] Load a .gltf file in the browser
- [ ] Render with PBR materials at 60fps
- [ ] Orbit/pan/zoom camera navigation
- [ ] Object list in outliner
- [ ] Property display for selected object
- [ ] Works in Chrome, Firefox, Safari (WebGPU)
- [ ] WASM bundle < 5MB gzipped

---

## Phase 2: EDITOR (Scene Assembly)

**Goal**: Add, delete, transform, and arrange objects.

### Sprint 2.1 — Operators & Undo
| Task | Status | Notes |
|------|--------|-------|
| Operator trait + registry (inventory) | [ ] | `trait Operator`, `poll/execute` |
| Undo/redo stack with bincode snapshots | [ ] | Memory-limited |
| Keyboard shortcut system | [ ] | Configurable bindings |
| Context struct (active object, mode, area) | [ ] | |

### Sprint 2.2 — Object Manipulation
| Task | Status | Notes |
|------|--------|-------|
| Add primitive operators (cube, sphere, plane, etc.) | [ ] | Via operator system |
| Delete object operator | [ ] | With undo |
| Transform gizmos (translate, rotate, scale) | [ ] | 3D gizmo rendering |
| Object picking (ray-cast from mouse) | [ ] | GPU picking or CPU ray-cast |
| Multi-select (shift-click, box select) | [ ] | Selection state in scene |
| Duplicate operator | [ ] | Deep copy with new ID |
| Parent/child hierarchy | [ ] | Set parent, clear parent |
| Snap to grid / increment | [ ] | |

### Sprint 2.3 — Materials & Textures
| Task | Status | Notes |
|------|--------|-------|
| Material editor panel (PBR properties) | [ ] | Color, metallic, roughness sliders |
| Material assignment to objects | [ ] | Material slots |
| Texture loading (PNG, JPEG, HDR) | [ ] | Via imbuf equivalent |
| UV display in viewport | [ ] | Texture-mapped rendering |
| Multiple materials per mesh | [ ] | Material indices on faces |
| Environment/HDRI background | [ ] | IBL for reflections |

### Sprint 2.4 — Scene I/O & Collaboration
| Task | Status | Notes |
|------|--------|-------|
| Save/load scene (JSON format) | [ ] | File System Access API |
| Auto-save to OPFS (browser storage) | [ ] | Periodic background save |
| Export to glTF | [ ] | |
| Edge worker: scene sync API | [ ] | Cloudflare Workers |
| Basic multiplayer (presence only) | [ ] | See who's viewing |

**Phase 2 Exit Criteria**:
- [ ] Add/delete/transform objects with undo/redo
- [ ] Material editing with real-time viewport preview
- [ ] Save/load projects to local filesystem
- [ ] Transform gizmos working
- [ ] Object picking and selection

---

## Phase 3: MODELER (Mesh Editing)

**Goal**: Edit mesh topology — the core of 3D modeling.

### Sprint 3.1 — EditMesh (SlotMap Half-Edge)
| Task | Status | Notes |
|------|--------|-------|
| EditMesh struct with SlotMap arena | [ ] | Vert/Edge/Loop/Face keys |
| Disk cycle operations | [ ] | Edges around vertex |
| Radial cycle operations | [ ] | Faces around edge |
| Mesh ↔ EditMesh conversion | [ ] | Enter/exit edit mode |
| CustomData layer storage | [ ] | UVs, vertex colors, weights |

### Sprint 3.2 — Mesh Operators
| Task | Status | Notes |
|------|--------|-------|
| Select vert/edge/face (picking) | [ ] | Component-level selection |
| Extrude | [ ] | Faces, edges, vertices |
| Loop cut | [ ] | Insert edge loops |
| Bevel | [ ] | Edge/vertex bevel |
| Inset faces | [ ] | |
| Merge vertices | [ ] | By distance, at center |
| Fill / Grid fill | [ ] | |
| Knife tool | [ ] | |
| Proportional editing | [ ] | Falloff-based transform |

### Sprint 3.3 — Modifiers
| Task | Status | Notes |
|------|--------|-------|
| Modifier trait + stack evaluation | [ ] | Chain of GeometrySet |
| Subdivision Surface | [ ] | Catmull-Clark (must implement) |
| Mirror modifier | [ ] | |
| Array modifier | [ ] | |
| Boolean modifier | [ ] | Via csgrs or custom |
| Solidify modifier | [ ] | |
| Decimate modifier | [ ] | |
| Modifier stack UI panel | [ ] | Add, remove, reorder, toggle |

### Sprint 3.4 — Geometry Nodes (Basic)
| Task | Status | Notes |
|------|--------|-------|
| Node graph data structure (petgraph) | [ ] | |
| Node type registration system | [ ] | |
| Visual node editor UI (Svelte) | [ ] | Canvas-based node drawing |
| Basic math/vector/geometry nodes | [ ] | |
| Attribute system (named data layers) | [ ] | |
| Mesh primitive generation nodes | [ ] | |

**Phase 3 Exit Criteria**:
- [ ] Enter/exit edit mode
- [ ] Basic mesh editing operations (extrude, loop cut, bevel)
- [ ] At least 5 working modifiers
- [ ] Modifier stack with non-destructive workflow

---

## Phase 4: ANIMATOR

### Key Sprints
- Keyframe insertion and timeline UI
- FCurve evaluation (Bezier interpolation)
- Dope sheet and graph editor
- Object constraints (Track To, Copy Location, etc.)
- Armature/bone system (basic)
- Shape keys
- Animation playback with depsgraph re-evaluation

---

## Phase 5: RENDERER

### Key Sprints
- EEVEE-equivalent real-time renderer (shadows, AO, bloom, SSR)
- Server-side Cycles rendering via edge worker → GPU server
- Render result streaming (progressive refinement)
- Compositor (basic post-processing nodes)
- Render settings panel

---

## Phase 6: FULL DCC (Ongoing)

### Feature Areas
- Sculpting (multires, dynamic topology)
- UV editing
- Texture painting
- Particle systems
- Physics simulation (rigid body, cloth, fluid)
- Video sequence editor
- Python scripting (via Pyodide or custom)
- Full node editors (shader, compositor)
- Asset browser
- VR/XR support (WebXR)

---

## Risk Registry

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| WASM bundle too large (>20MB) | High | Medium | Tree-shaking, lazy loading, streaming |
| Half-edge mesh in Rust too slow | High | Low | SlotMap is fast; benchmark early |
| WebGPU browser gaps | Medium | Low | Canvas2D/WebGL2 fallback |
| Catmull-Clark subdivision hard to implement | Medium | Medium | Start simple (loop subdivision), iterate |
| js↔wasm boundary perf bottleneck | High | Medium | Batch operations, SharedArrayBuffer |
| CADmium-style project failure | High | Medium | Phase-based delivery, each phase is standalone |
| Scope creep (Blender has everything) | High | High | Strict phase boundaries, user research |
| Team skill gap (Rust + WASM + WebGPU) | Medium | Medium | Comprehensive docs, mentoring |

---

## Dependency Graph (Phase Dependencies)

```
Phase 1 (Viewer) ──────► Phase 2 (Editor) ──────► Phase 3 (Modeler)
                                │                        │
                                ▼                        ▼
                         Phase 4 (Animator)       Phase 5 (Renderer)
                                │                        │
                                └────────┬───────────────┘
                                         ▼
                                  Phase 6 (Full DCC)

Cross-cutting concerns (all phases):
  - AI/MCP integration (progressive tool exposure)
  - Edge worker (progressive feature support)
  - Performance optimization (continuous)
  - Testing (continuous)
```

---

## Crate Dependency Map (Target Architecture)

```
blender-derive (proc macro)
    │
    ▼
blender-core (data types, traits, ID system)
    │
    ├──► blender-mesh (EditMesh, half-edge, operators)
    ├──► blender-depsgraph (evaluation DAG)
    ├──► blender-modifiers (modifier traits + implementations)
    ├──► blender-nodes (node graph, geometry nodes)
    ├──► blender-animation (keyframes, FCurves, constraints)
    │
    ▼
blender-gpu (wgpu renderer, shaders, camera)
    │
    ├──► blender-draw (EEVEE-like engine, viewport overlays)
    │
    ▼
blender-io (file I/O: .blend parser, glTF, USD)
    │
    ▼
blender-wasm (wasm-bindgen API surface)
    │
    ▼
packages/ui (SvelteKit frontend)
packages/edge-worker (Cloudflare Workers)
packages/mcp-server (AI tools)
```

**Key difference from Blender**: No circular dependencies.
`blender-core` is the foundation; everything depends down, never up.
The WASM layer is a thin facade over the Rust crates.

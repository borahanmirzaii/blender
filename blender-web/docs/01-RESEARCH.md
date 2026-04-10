# Blender Web — Research & Analysis Report

## 1. Blender Codebase Analysis

### Scale
- **~2.96 million LOC** across **~7,800 source files**
- **55+ modules** with complex interdependencies
- **24 modules minimum** required for load-evaluate-render pipeline

### Top 10 Modules by Code Size
| Module | LOC | Files | Purpose |
|--------|-----|-------|---------|
| editors | 792,710 | 1,122 | UI editors (48 sub-libraries) |
| blenkernel | 370,384 | 557 | Core data management — THE hub |
| cycles | 206,747 | 755 | Production ray-tracer |
| makesrna | 196,454 | 126 | Runtime property reflection |
| blenlib | 191,673 | 496 | Utility library (math, containers) |
| draw | 168,930 | 415 | Viewport rendering engines |
| nodes | 145,981 | 655 | Geometry/Shader/Compositor nodes |
| gpu | 107,396 | 420 | GPU abstraction (GL/VK/Metal) |
| python | 89,753 | 198 | Python scripting bindings |
| freestyle | 85,137 | 525 | NPR rendering |

### Circular Dependency Problem
`blenkernel` has **bidirectional dependencies with 13+ modules**:
```
blenkernel <-> depsgraph    (BKE:85 files use DEG, DEG:44 use BKE)
blenkernel <-> blenloader   (BKE:74 files use BLO, BLO:29 use BKE)
blenkernel <-> nodes        (BKE:27 files use NOD, Nodes:339 use BKE)
blenkernel <-> draw         (BKE:7 files use DRW, Draw:124 use BKE)
blenkernel <-> render       (BKE:63 files use RE, Render:19 use BKE)
blenkernel <-> modifiers    (BKE:31 files use MOD)
blenkernel <-> bmesh        (BKE:9 files use bmesh, BMesh:36 use BKE)
blenkernel <-> sequencer    (BKE:5 files use SEQ, SEQ:31 use BKE)
blenkernel <-> animrig      (BKE:20 files use ANIM)
blenkernel <-> windowmanager (BKE:12 files use WM, WM:42 use BKE)
blenkernel <-> gpu           (BKE:12 files use GPU, GPU:34 use BKE)
blenkernel <-> imbuf         (BKE:44 files use IMB, IMB:13 use BKE)
```

**Implication**: We CANNOT simply port Blender module-by-module. We must
redesign the dependency graph to eliminate circular dependencies by
introducing clean interface boundaries.

---

## 2. Blender's Core Design Patterns (Actual Code Analysis)

### 2.1 DNA/RNA — Two-Layer Data Model

**DNA** (makesdna) — C struct definitions that serialize to .blend files:
```c
// DNA_object_types.h
struct Object {
    ID id;                    // Base data-block (all IDs inherit this)
    float loc[3], rot[4], scale[3];  // Transform
    Object *parent;           // Hierarchy
    ModifierData *modifiers;  // Modifier stack
    ObjectRuntime *runtime;   // Non-serialized computed data
};
```

**RNA** (makesrna) — Reflection system exposing DNA to Python/UI:
```c
// PointerRNA wraps any DNA struct with type metadata
struct PointerRNA {
    ID *owner_id;
    StructRNA *type;        // Type metadata (fields, getters, setters)
    void *data;             // Actual C struct pointer
};
```

**Key insight**: RNA is a **runtime property registry** with getter/setter
callbacks, validation, and automatic Python binding generation. ~196K LOC
dedicated to this reflection layer.

### 2.2 Depsgraph — Lazy-Evaluating DAG

```
Operation evaluation order (150+ operation codes):
  Transform: INIT → LOCAL → PARENT → CONSTRAINTS → EVAL → FINAL
  Geometry:  GEOMETRY_EVAL_INIT → MODIFIER → GEOMETRY_EVAL → DONE
  Pose:      POSE_INIT → IK_SOLVER → POSE_DONE
  Animation: ANIMATION_ENTRY → ANIMATION_EVAL → ANIMATION_EXIT
```

Core patterns:
- **Copy-on-Write**: `ID_orig` → `ID_cow` (evaluated copy)
- **Lazy evaluation**: Only dirty nodes are re-evaluated
- **Topological sort**: Pre-computed order for parallel execution
- **Tag system**: `DEG_id_tag_update()` marks dirty, propagates to dependents

### 2.3 Operators — Command Pattern

```c
struct wmOperatorType {
    char idname[64];           // "mesh.select_all"
    int (*exec)(bContext *, wmOperator *);
    int (*modal)(bContext *, wmOperator *, wmEvent *);
    bool (*poll)(bContext *);  // Can operator run in this context?
    int flag;                  // OPTYPE_REGISTER | OPTYPE_UNDO
};
```

Every user action (move vertex, add object, change material) is an Operator
that can be undone, redone, and recorded for scripting.

### 2.4 BMesh — Half-Edge Mesh

```c
struct BMVert { float co[3]; float no[3]; BMEdge *e; };
struct BMEdge { BMVert *v1, *v2; BMLoop *l; };
struct BMLoop { BMVert *v; BMEdge *e; BMFace *f; BMLoop *next, *prev; };
struct BMFace { BMLoop *l_first; int len; };
```

- **Disk cycles**: edges around a vertex
- **Radial cycles**: faces around an edge
- **BLI_mempool**: O(1) allocation from pre-allocated pools

### 2.5 Event System — Chain of Responsibility

```
Event Handler Chain:
  wmEventHandler_Keymap  → keybindings → operators
  wmEventHandler_Op      → modal operator (brushes, transforms)
  wmEventHandler_UI      → button/panel input
  wmEventHandler_Gizmo   → transform gizmos
  wmEventHandler_Dropbox → drag-drop
```

### 2.6 GPU Abstraction — Bridge Pattern

```
GPUBackendType { OpenGL, Vulkan, Metal, None }
GPU objects: Context, Shader, Batch, VertBuf, IndexBuf, FrameBuffer, Texture
Pattern: Abstract interface → concrete backend (GL/VK/Metal)
Deferred creation: actual GPU resources created lazily on first use
```

### 2.7 Render Engines — Strategy + Registry

```c
struct RenderEngineType {
    char idname[64];
    void (*update)(RenderEngine *, Main *, Depsgraph *);
    void (*render)(RenderEngine *, Depsgraph *);
    void (*draw)(RenderEngine *, const bContext *, Depsgraph *);
};
```
Pluggable: EEVEE, Cycles, Workbench, or Python-defined external engines.

### 2.8 Node System — LazyFunction + Fields

```
LazyFunction {
    inputs, outputs,
    execute(Executor &)     // Topological evaluation with backtracking
}

Fields: function objects evaluated at each geometry element
Socket inference: if any input is a field, outputs become fields
```

### 2.9 Modifier Stack — Chain + GeometrySet

```c
struct ModifierTypeInfo {
    ModifierTypeType type;  // OnlyDeform | Constructive | Nonconstructive
    void (*deformVerts)(...);    // Position-only modification
    GeometrySet (*applyModifier)(...);  // Full geometry modification
};
```

GeometrySet: modern multi-type container (Mesh, Curve, PointCloud, Volume)
flowing through the modifier chain.

### 2.10 File I/O — Custom Binary + Versioning

```
.blend = serialized memory dump with:
  - Struct DNA (type definitions embedded in file)
  - Pointer relinking on load (address maps)
  - Forward/backward compatibility via versioning functions
  - Same format used for undo snapshots
```

---

## 3. Ecosystem Research Findings

### 3.1 Rust 3D/Graphics Crates

| Crate | Status | WASM | Notes |
|-------|--------|------|-------|
| **wgpu** v29 | Production | Yes (WebGPU/WebGL2) | Used by Firefox, Deno. THE GPU abstraction |
| **Bevy** v0.18 | Active | Yes (15-30MB) | Full ECS engine, too opinionated for us |
| **Fyrox** v0.36 | Active | Limited | Has visual editor, closest to Unity-in-Rust |
| **glam** v0.32 | Stable | Yes | Fast SIMD math. Used by Bevy/wgpu |
| **nalgebra** v0.34 | Stable | Yes | Generic math. Dimforge ecosystem |
| **petgraph** | Stable | Yes | Graph data structures (for depsgraph) |
| **slotmap** | Stable | Yes | Arena allocation (for mesh data) |
| **serde** | Stable | Yes | Serialization framework |
| **csgrs** | Active | Yes | CSG booleans (float-based, not exact) |
| **truck** | Active | Yes (truck-js) | B-rep CAD kernel |
| **tri-mesh** | Active | Yes | Half-edge mesh (basic) |

### 3.2 Critical Gaps in Rust Ecosystem

| Need | Status | Mitigation |
|------|--------|------------|
| Production half-edge mesh editor | **Missing** | Build with arena allocation (slotmap) |
| Catmull-Clark subdivision | **Missing** | Must implement from scratch |
| Exact arithmetic CSG | **Missing** | Use csgrs (float) or port CGAL |
| Node graph evaluation framework | **Missing** | Build on petgraph |
| Property reflection/RNA equivalent | **Missing** | Build with proc macros + inventory crate |
| .blend file parser | **Missing** | Build incrementally |
| Undo system | **Missing** | Build with serde serialization |

### 3.3 WASM/WASI Constraints

| Feature | Status (2026) |
|---------|---------------|
| Memory | 4GB default, Memory64 in WASM 3.0 (rolling out) |
| Threads | Standardized in WASM 3.0 (Sept 2025). SharedArrayBuffer required |
| SIMD | 128-bit standardized in WASM 3.0 |
| Performance | ~90-95% of native for optimized code |
| WebGPU | Shipped in Chrome, Firefox, Safari, Edge (desktop) |
| WASI filesystem | Requires shim libraries in browser (browser_wasi_shim) |
| Component Model | Server-side production-ready; browser still maturing |
| Bundle size | Bevy: 15-30MB WASM. Must plan for streaming/lazy loading |

### 3.4 WebGPU Browser Support (2026)

| Browser | Status |
|---------|--------|
| Chrome 113+ | Shipped (Win/Mac/ChromeOS/Android 12+) |
| Firefox 141+ | Shipped (Win). Mac Apple Silicon v145 |
| Safari 26+ | Shipped (macOS/iOS/iPadOS/visionOS) |
| Edge | Shipped (same as Chrome) |

### 3.5 Prior Art — Lessons Learned

**Figma** (most relevant):
- C++ rendering engine compiled to WASM via Emscripten
- TypeScript+React for UI, C++ WASM for canvas/document
- ~15M lines C++ in rendering core
- 3x load time improvement after WASM switch
- Migrating from WebGL to WebGPU

**AutoCAD Web**:
- 15M lines C++ → WASM via Emscripten
- Proves massive C++ codebases can run in browser
- 3D support still has gaps vs desktop

**CADmium** (Rust+WASM+Svelte — **discontinued**):
- Rust backend (Truck B-rep kernel) → WASM
- SvelteKit + Threlte (three.js) frontend
- Tauri for native desktop
- **Failed to reach stable state** — important cautionary tale

**Key lesson**: Nobody has built Blender-in-browser. The closest successes
(Figma, AutoCAD) compiled existing C++ to WASM. A ground-up Rust rewrite
is higher risk but offers cleaner architecture.

---

## 4. AI/MCP for 3D — State of the Art

**Existing MCP servers**:
- `blender-mcp` — TCP socket bridge: Claude → JSON commands → Blender addon
- `meshy-ai-mcp-server` — Text/image-to-3D via Meshy API
- SketchUp MCP, Rhino 3D MCP — natural language modeling

**Text-to-3D models**:
- Hunyuan3D v3.0 (Tencent, Apache 2.0) — production-ready, 6GB VRAM minimum
- 30-120 second generation, ~70-80% first-gen usability

**Gap**: No system exists where an AI agent renders 3D, observes the result
visually, and iterates in a closed feedback loop. This is a novel opportunity.

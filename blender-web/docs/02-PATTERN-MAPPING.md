# Blender Web — Pattern Mapping: C++ → Rust

## Design Philosophy

Blender's architecture evolved organically over 30 years in C/C++. We cannot
simply transliterate it. Instead, we map each **architectural concern** to
Rust idioms that solve the same problem with Rust's strengths (ownership,
traits, enums, proc macros) while avoiding its weaknesses (graph structures,
shared mutability).

---

## 1. DNA/RNA → Rust Reflection System

### Problem
Blender needs a data model that is:
- Serializable to a binary file format
- Inspectable at runtime (UI property panels, Python API)
- Versionable (old files load in new versions)
- Animatable (any property can be keyframed)

### Blender's Approach (C)
```
DNA: C structs defined in headers → compiled into SDNA block in .blend files
RNA: Hand-written getter/setter registration → PointerRNA + StructRNA
```

### Rust Approach: Proc Macro + Trait-Based Reflection

```rust
// Derive macro generates: serialization, property registry, UI hints
#[derive(BlenderData)]
#[bw_type(id_type = "OB")]
pub struct Object {
    #[bw_prop(name = "Location", subtype = "XYZ", animatable)]
    pub location: Vec3,

    #[bw_prop(name = "Scale", subtype = "XYZ", animatable, default = "Vec3::ONE")]
    pub scale: Vec3,

    #[bw_prop(name = "Visible", description = "Object visibility")]
    pub visible: bool,

    #[bw_prop(skip_serialize)]  // Runtime-only data
    pub runtime: ObjectRuntime,
}
```

**Implementation components:**
| Component | Rust Crate/Approach |
|-----------|-------------------|
| Struct definition | Plain Rust structs |
| Serialization | `serde` + custom binary format |
| Property metadata | `#[derive(BlenderData)]` proc macro |
| Runtime registry | `inventory` crate (linker-based static registration) |
| Type introspection | `TypeId` + trait objects (`dyn PropertyAccess`) |
| UI generation | Property iterator → Svelte component generation |
| Animation binding | Property path strings ("object.location.x") |

**Key trait:**
```rust
pub trait BlenderData: Send + Sync + 'static {
    fn type_info() -> &'static TypeInfo;
    fn properties() -> &'static [PropertyDef];
    fn get_property(&self, path: &str) -> Option<PropertyValue>;
    fn set_property(&mut self, path: &str, value: PropertyValue) -> Result<()>;
}
```

### Why This Works
- Proc macro generates boilerplate at compile time (zero runtime cost)
- `inventory` crate enables global type registry without manual registration
- `serde` provides the serialization foundation
- Trait-based instead of pointer-based (no `void *data` reinterpretation)

---

## 2. Depsgraph → Petgraph + Topological Evaluation

### Problem
Scene evaluation must:
- Track dependencies between objects, modifiers, constraints, animations
- Only re-evaluate what changed (lazy/incremental)
- Support parallel evaluation of independent branches
- Handle multiple simultaneous evaluations (viewport + render)

### Blender's Approach
```
Custom DAG with OperationNode, ComponentNode, IDNode
150+ operation codes, topological sort, Copy-on-Write
```

### Rust Approach: Petgraph + Arena + COW

```rust
use petgraph::stable_graph::StableDiGraph;

pub struct Depsgraph {
    graph: StableDiGraph<EvalNode, Relation>,
    // Node lookup by ID + component type
    id_to_nodes: HashMap<DataBlockId, SmallVec<[NodeIndex; 8]>>,
    // Dirty tracking
    dirty_nodes: BitVec,
    // Evaluation order (cached topological sort)
    eval_order: Vec<NodeIndex>,
    // COW: evaluated copies stored separately
    eval_store: EvalStore,
}

pub enum EvalNode {
    Transform(TransformOps),
    Geometry(GeometryOps),
    Animation(AnimationOps),
    // ... 50+ node types mapped from Blender's NodeType enum
}

pub enum TransformOps {
    Init, Local, Parent, Constraints, Eval, Final
}
```

**Key design decisions:**
| Decision | Rationale |
|----------|-----------|
| `petgraph::StableDiGraph` | Stable indices survive node removal |
| `BitVec` for dirty flags | Cache-friendly, fast bulk operations |
| COW via `Arc<T>` | Multiple readers, single writer per eval |
| Cached topo sort | Recompute only when graph topology changes |
| `rayon` for parallel eval | Fork-join parallelism on independent branches |

---

## 3. BMesh → Arena-Allocated Half-Edge Mesh

### Problem
Mesh editing requires:
- O(1) adjacency queries (vertices around vertex, faces around edge)
- Efficient topology modification (split, merge, extrude)
- Support for N-gons (arbitrary polygon count)
- Custom data layers (UVs, colors, weights per element)

### Blender's Approach
```c
// Intrusive linked lists with raw pointers
BMVert { co[3], no[3], *e }      // disk cycle entry
BMEdge { *v1, *v2, *l }          // radial cycle entry
BMLoop { *v, *e, *f, *next, *prev, *radial_next, *radial_prev }
BMFace { *l_first, len }
// BLI_mempool for O(1) allocation
```

### Rust Approach: SlotMap Arena + Indices

```rust
use slotmap::{SlotMap, new_key_type};

new_key_type! {
    pub struct VertKey;
    pub struct EdgeKey;
    pub struct LoopKey;
    pub struct FaceKey;
}

pub struct EditMesh {
    pub verts: SlotMap<VertKey, Vert>,
    pub edges: SlotMap<EdgeKey, Edge>,
    pub loops: SlotMap<LoopKey, Loop>,
    pub faces: SlotMap<FaceKey, Face>,
    // Custom data layers (equivalent to CustomData)
    pub vert_layers: LayerStorage<VertKey>,
    pub edge_layers: LayerStorage<EdgeKey>,
    pub loop_layers: LayerStorage<LoopKey>,
    pub face_layers: LayerStorage<FaceKey>,
}

pub struct Vert {
    pub co: Vec3,
    pub normal: Vec3,
    pub edge: Option<EdgeKey>,  // Disk cycle entry
}

pub struct Edge {
    pub v1: VertKey,
    pub v2: VertKey,
    pub loop_: Option<LoopKey>,  // Radial cycle entry
    pub v1_disk_next: Option<EdgeKey>,
    pub v1_disk_prev: Option<EdgeKey>,
    pub v2_disk_next: Option<EdgeKey>,
    pub v2_disk_prev: Option<EdgeKey>,
}

pub struct Loop {
    pub vert: VertKey,
    pub edge: EdgeKey,
    pub face: FaceKey,
    pub next: LoopKey,
    pub prev: LoopKey,
    pub radial_next: LoopKey,
    pub radial_prev: LoopKey,
}

pub struct Face {
    pub loop_first: LoopKey,
    pub len: u32,
}
```

### Why SlotMap Over Rc<RefCell<T>>
| Approach | Performance | Safety | Ergonomics |
|----------|------------|--------|------------|
| `Rc<RefCell<T>>` | Slow (refcount overhead) | Runtime panics | Awkward borrows |
| Raw pointers (`unsafe`) | Fast | Unsafe | C-like, defeats purpose |
| **SlotMap + indices** | **Fast (arena)** | **Compile-time safe** | **Natural in Rust** |
| ECS (hecs/bevy_ecs) | Cache-friendly | Safe | Wrong abstraction for topology |

SlotMap gives us:
- O(1) insert/remove with stable keys
- Generational indices (detect use-after-free)
- Dense storage (cache-friendly iteration)
- No lifetime/borrow complications

---

## 4. Operators → Command Pattern with Undo

### Blender's Approach
```c
wmOperatorType { idname, exec(), modal(), poll(), OPTYPE_UNDO }
Undo = serialize entire Main to MemFile
```

### Rust Approach: Trait + Undo Stack

```rust
pub trait Operator: Send + Sync {
    fn id(&self) -> &str;
    fn poll(&self, ctx: &Context) -> bool;
    fn execute(&self, ctx: &mut Context) -> OperatorResult;

    // Optional: for modal operators (brushes, transforms)
    fn modal(&self, _ctx: &mut Context, _event: &Event) -> OperatorResult {
        OperatorResult::Cancelled
    }

    // Undo support
    fn undo_type(&self) -> UndoType { UndoType::Full }
}

pub enum UndoType {
    None,                    // No undo (e.g., view navigation)
    Full,                    // Snapshot entire scene (Blender's approach)
    Incremental(UndoDiff),   // Store only the diff (more efficient)
}

// Undo stack with scene snapshots
pub struct UndoStack {
    steps: Vec<UndoStep>,
    current: usize,
    max_memory: usize,  // Limit total undo memory
}

pub struct UndoStep {
    name: String,
    // Serialized scene state (using serde + bincode for speed)
    snapshot: Vec<u8>,
    timestamp: Instant,
}
```

**Undo strategy:**
- Full snapshots for complex operations (like Blender)
- `bincode` serialization (faster than JSON, compact binary)
- Memory-limited undo stack with oldest entries evicted
- Future: incremental undo via structural diffing

---

## 5. Event System → Svelte + WASM Bridge

### Blender's Approach
```
GHOST (OS events) → wmEvent → Handler chain → Operator invocation
```

### Rust+Svelte Approach: Split Responsibility

```
Browser DOM events → Svelte event handlers → WASM bridge → Rust logic
```

| Layer | Responsibility | Technology |
|-------|---------------|------------|
| Browser | Raw input (mouse, keyboard, touch) | DOM events |
| Svelte | UI events (button clicks, panel interaction) | Svelte event directives |
| Bridge | Translate to Rust event types | wasm-bindgen |
| Rust | 3D viewport interaction (orbit, select, transform) | Rust event handler chain |

```rust
// Events that cross the WASM boundary
#[derive(Serialize, Deserialize)]
pub enum ViewportEvent {
    MouseMove { x: f32, y: f32, buttons: u8 },
    MouseDown { x: f32, y: f32, button: MouseButton },
    MouseUp { x: f32, y: f32, button: MouseButton },
    Wheel { delta: f32 },
    KeyDown { code: KeyCode, modifiers: Modifiers },
    KeyUp { code: KeyCode },
}

// Svelte → WASM: minimal data crossing the boundary
// Rust processes 3D interaction logic (picking, gizmos, tools)
// Result → Svelte via callback or shared state
```

**Key principle**: DOM events stay in JS/Svelte land. Only 3D viewport
interaction crosses into WASM. This minimizes JS↔WASM boundary crossing.

---

## 6. GPU Abstraction → wgpu (Already Solved)

### Blender's Approach
Custom `GPU_*` API abstracting OpenGL, Vulkan, Metal

### Rust Approach: Use wgpu Directly

wgpu already provides exactly what Blender's GPU layer does:
- Same API across Vulkan, Metal, D3D12, OpenGL, **and WebGPU**
- Same Rust code runs native AND in browser (WASM)
- Shader translation via Naga (WGSL → SPIR-V/MSL/HLSL/GLSL)
- Production-proven (Firefox, Deno, Bevy)

**No custom GPU abstraction needed** — wgpu IS the abstraction.

---

## 7. Render Engines → Trait-Based Plugin System

### Blender's Approach
```c
RenderEngineType { idname, update(), render(), draw() }
```

### Rust Approach

```rust
pub trait RenderEngine: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> EngineCapabilities;

    // Scene changed — rebuild internal state
    fn update(&mut self, scene: &Scene, depsgraph: &Depsgraph);

    // Produce a final render
    fn render(&mut self, scene: &Scene) -> RenderResult;

    // Viewport draw (real-time)
    fn viewport_draw(&mut self, scene: &Scene, camera: &ViewportCamera)
        -> wgpu::CommandBuffer;
}

// Registry for engine discovery
inventory::collect!(Box<dyn RenderEngineFactory>);

pub trait RenderEngineFactory: Send + Sync {
    fn create(&self, device: &wgpu::Device) -> Box<dyn RenderEngine>;
    fn name(&self) -> &str;
}
```

---

## 8. Node System → Typed Graph + LazyFunction

### Blender's Approach
```
LazyFunction with Field system, socket type inference,
topological evaluation with backtracking
```

### Rust Approach

```rust
pub struct NodeGraph {
    nodes: SlotMap<NodeKey, Node>,
    links: Vec<Link>,
    graph: StableDiGraph<NodeKey, ()>,  // For evaluation order
}

pub struct Node {
    pub type_id: NodeTypeId,
    pub inputs: Vec<Socket>,
    pub outputs: Vec<Socket>,
    pub properties: HashMap<String, PropertyValue>,
}

// Type-safe socket system using Rust enums
pub enum SocketValue {
    Float(f32),
    Vector(Vec3),
    Color(Vec4),
    Geometry(GeometrySet),
    Field(Box<dyn FieldFunction>),
    Shader(ShaderClosure),
    String(String),
    Bool(bool),
    Int(i32),
}

// Field evaluation (vectorized per-element computation)
pub trait FieldFunction: Send + Sync {
    fn evaluate(&self, domain: &AttributeDomain, indices: Range<usize>,
                output: &mut [f32]);
}

// Node type registration
pub trait NodeType: Send + Sync {
    fn declaration(&self) -> NodeDeclaration;
    fn execute(&self, inputs: &[SocketValue], outputs: &mut [SocketValue]);
}

inventory::collect!(Box<dyn NodeType>);
```

---

## 9. Modifier Stack → Iterator Chain + GeometrySet

### Rust Approach

```rust
pub enum GeometryComponent {
    Mesh(EditMesh),         // or evaluated Mesh
    Curve(CurveData),
    PointCloud(PointCloud),
    Volume(VolumeData),
    Instances(InstanceData),
}

pub struct GeometrySet {
    components: SmallVec<[GeometryComponent; 2]>,
}

pub trait Modifier: Send + Sync {
    fn name(&self) -> &str;
    fn modifier_type(&self) -> ModifierType;
    fn apply(&self, input: GeometrySet, ctx: &ModifierContext) -> GeometrySet;

    // Deform-only modifiers can skip full geometry copy
    fn deform_only(&self) -> bool { false }
    fn deform(&self, positions: &mut [Vec3], _ctx: &ModifierContext) {}
}

// Modifier stack evaluation
pub fn evaluate_modifiers(
    base_geometry: GeometrySet,
    modifiers: &[Box<dyn Modifier>],
    ctx: &ModifierContext,
) -> GeometrySet {
    modifiers.iter()
        .filter(|m| m.enabled())
        .fold(base_geometry, |geo, modifier| {
            if modifier.deform_only() {
                let mut geo = geo;
                if let Some(mesh) = geo.mesh_mut() {
                    modifier.deform(mesh.positions_mut(), ctx);
                }
                geo
            } else {
                modifier.apply(geo, ctx)
            }
        })
}
```

---

## 10. File I/O → Versioned Binary + JSON Interchange

### Approach

| Format | Use Case | Crate |
|--------|----------|-------|
| `.bweb` (custom binary) | Primary save format | `bincode` + versioning |
| `.json` | Interchange, debugging | `serde_json` |
| `.blend` (read-only) | Import from Blender | Custom parser (incremental) |
| `.gltf/.glb` | 3D interchange | `gltf` crate |
| `.usd` | Production pipeline | `usd-rs` (if available) |

```rust
// Versioned file format
pub struct FileHeader {
    magic: [u8; 4],        // "BWEB"
    version: [u16; 3],     // major.minor.patch
    pointer_size: u8,      // 4 or 8
    endianness: u8,        // 0=little, 1=big
    checksum: u64,
}

// Version migration
pub fn migrate(data: &mut Scene, from_version: Version, to_version: Version) {
    // Chain of migration functions, like Blender's versioning_*.cc
    for migration in MIGRATIONS.iter() {
        if migration.from <= from_version && migration.to <= to_version {
            (migration.apply)(data);
        }
    }
}
```

---

## Summary: Pattern Mapping Table

| Blender Pattern | Blender Impl | Rust Equivalent | Key Crate |
|----------------|-------------|-----------------|-----------|
| DNA (data structs) | C structs + SDNA | Rust structs + `#[derive(BlenderData)]` | proc-macro, serde |
| RNA (reflection) | StructRNA + PropertyRNA | Trait `BlenderData` + inventory registry | inventory |
| Depsgraph | Custom DAG | `petgraph::StableDiGraph` + BitVec dirty | petgraph |
| BMesh (half-edge) | Raw pointers + mempool | `SlotMap` arena + typed keys | slotmap |
| CustomData layers | void* + type enum | `LayerStorage<K>` generic over element key | — |
| Operators | wmOperatorType + func ptrs | `trait Operator` + inventory | inventory |
| Undo | MemFile serialized snapshot | `bincode` snapshot + undo stack | bincode |
| Event system | GHOST + handler chain | DOM → Svelte → wasm-bindgen → Rust | wasm-bindgen |
| GPU abstraction | GPU_* API (GL/VK/Metal) | **wgpu** (already cross-platform + WASM) | wgpu |
| Render engines | RenderEngineType registry | `trait RenderEngine` + inventory | inventory, wgpu |
| Node system | LazyFunction + Fields | `NodeGraph` + `trait NodeType` | petgraph |
| Modifier stack | ModifierTypeInfo chain | `trait Modifier` + fold evaluation | — |
| File I/O | Custom binary + SDNA | `bincode` + versioned migrations | bincode, serde |
| Context passing | `bContext *C` everywhere | `struct Context { &Scene, &Area, ... }` | — |
| Memory pools | BLI_mempool | `SlotMap` / `bumpalo` | slotmap, bumpalo |
| ID system | `struct ID` base | `trait DataBlock` + `DataBlockId` | — |
| Animation | AnimData + FCurve | `AnimationData` + spline evaluation | — |

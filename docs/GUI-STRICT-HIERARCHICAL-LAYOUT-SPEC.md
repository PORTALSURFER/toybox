# Strict Hierarchical Slot-Based Layout System

This document defines the target layout model for Patchbay declarative UIs and
maps current implementation status to that target.

## 0) Goals

- Deterministic, repeatable layout for the same tree + root frame.
- Responsive to root-size changes without global screen placement by children.
- Composition-first: containers place children; widgets express preferences.
- Explicit overflow/underflow behavior.
- Real-time-safe performance via deterministic math and minimal recompute.

Non-goals:

- Freeform drag placement in the layout system itself.
- General 2D constraint solving across arbitrary node relationships.

## 1) Core Concepts

### 1.1 Node Types

- **Node**: every UI element in the declarative tree.
- **Widget node**: mostly-leaf rendering/interaction element with intrinsic size behavior.
- **Container node**: defines one or more slots and a placement algorithm.
- **Slot**: parent-owned child attachment point that carries child placement rules.

Hard rule:

- Every node is hosted by exactly one slot, except the root container itself.

### 1.2 Coordinate Space

- Layout is computed in local coordinates.
- Output is a resolved rectangle per node: `Rect { x, y, w, h }`.
- Nodes do not assign global positions; containers assign child rectangles.

## 2) Layout Data Model

### 2.1 Size Modes

Target size modes across layout surfaces:

- `Fixed(px)`
- `Fill(weight)`
- `Percent(p)` (optional when equivalent behavior can be represented via fill)
- `Intrinsic`
- `Aspect(ratio)` (optional)

All sizing is subject to constraints:

- `min_w`, `max_w`, `min_h`, `max_h`
- `padding` (container-owned)
- `margin` (slot-owned)

### 2.2 Alignment

- Main axis: `Start | Center | End | SpaceBetween | SpaceAround | SpaceEvenly`
- Cross axis: `Start | Center | End | Stretch`

### 2.3 Overflow Policy

Every container declares explicit behavior when content does not fit:

- `Clip` (default)
- `Scroll` (via viewport container/widget)
- `Wrap` (flow-like containers)
- `Ellipsize` (widget-level, typically text)
- `Shrink/Compress` (deterministic compression rules)

## 3) Deterministic Layout Algorithm

### 3.1 Two-Pass Model

Pass A (bottom-up measure):

- `measure(node, constraints) -> Size`
- Widgets resolve preferred/intrinsic size under constraints.
- Containers measure children via slot rules and compute preferred size.

Pass B (top-down layout):

- `layout(node, rect) -> placements`
- Containers receive final rect and assign child rects.
- Widgets consume assigned rects and do not reposition themselves.

### 3.2 Constraint Propagation

- Parent content rect yields child constraints after padding/margins.
- Containers clamp computed sizes to constraints.
- Intrinsic sizes are clamped as well.

### 3.3 Space Distribution (Row/Column/Flex-like)

Deterministic order:

1. Compute available main-axis space.
2. Resolve non-flex children.
3. Compute remainder.
4. Distribute remainder to `Fill(weight)` children.
5. Apply min/max clamps and iteratively re-distribute when clamps activate.
6. If overflow remains, apply explicit overflow policy.

### 3.4 Compression Policy

When `sum(min_sizes) > available`, compression order must be explicit and stable.

Recommended default:

1. Compress `Fill` children to min.
2. Compress `Intrinsic` children to min.
3. Compress `Fixed` children to min only when allowed.
4. If still not fitting, apply overflow policy.

## 4) Slot Semantics

### 4.1 Ownership

- Containers own slot definitions.
- Children cannot override slot ownership rules.

### 4.2 Slot Parameters (minimum)

Per slot target parameters:

- `size_main`: `Fixed | Fill(weight) | Intrinsic | Percent`
- `size_cross`: `Fixed | Fill | Intrinsic`
- min/max constraints
- margin
- optional alignment overrides

### 4.3 Single-slot vs Multi-slot Containers

- Single-slot examples: padding/alignment/aspect wrappers.
- Multi-slot examples: row/column/grid/list.
- Dynamic lists still attach children through container-owned slot entries.

## 5) Baseline Container Set

Target primitives:

- `PaddingBox` (1 slot)
- `AlignBox` (1 slot)
- `Row` / `Column` (multi-slot, flex-like)
- `Grid`
- `Stack`
- `ScrollView`
- Optional `Wrap/Flow`

Each container specifies:

- measure rules
- layout rules
- overflow behavior
- deterministic ordering

## 6) Widgets Contract

### 6.1 Intrinsics

Widgets may expose:

- `min_intrinsic(constraints)`
- `preferred(constraints)`
- optional baseline

Text widgets should define:

- wrapping mode
- ellipsis mode
- line-height policy

### 6.2 Rendering

- Widgets render inside final assigned rect.
- Out-of-rect rendering is only allowed by explicit overflow policy.

## 7) Responsiveness Requirements

- Root-size changes recompute layout deterministically.
- Sizing is relative/constraint-clamped.
- DPI scaling is applied at a controlled root boundary.
- Optional `SwitchLayout` container may select subtree variants by thresholds.

## 8) Determinism and Stability Rules

- Layout depends only on tree, slot params, widget intrinsics, and root rect.
- No randomness or frame-time dependence.
- Iterative flex/clamp logic has deterministic termination.
- Pixel rounding strategy is defined and consistently applied.
- Z-order is slot order unless explicit z-order fields are provided.

## 9) Invalid States and Diagnostics

Required behavior:

- Negative resolved sizes -> clamp to zero and record diagnostic.
- `min > max` contradictions -> normalize and record diagnostic.
- Overflow without explicit policy -> fallback `Clip` + diagnostic.
- Construction-time cycle prevention.

Debug visualization should support:

- container/slot bounds
- padding and margin overlays
- overflow indicators
- measured-vs-final deltas

## 10) Performance and Caching

### 10.1 Dirty Propagation

Track at least:

- `layout_dirty` (geometry-affecting)
- `measure_dirty` (intrinsic-affecting)

Recompute only affected subtrees.

### 10.2 Measure Cache Keys

Cache by:

- node id
- constraint key
- widget state version

### 10.3 Allocation Discipline

- Avoid per-frame deep allocations in layout passes.
- Prefer stable storage/id-based traversal where possible.

## 11) Suggested Minimal API Surface

- `NodeId`
- widget measure/paint contract
- container measure/layout contract
- slot params
- constraints
- layout result with overflow flags and diagnostics

## 12) Test Spec

### 12.1 Golden Layout Tests

Per container:

- multiple root sizes
- mixed fixed/fill/intrinsic inputs
- min/max constraints
- overflow cases

### 12.2 Property Tests

- Rect bounds invariants
- no NaN/negative sizes
- deterministic input/output behavior

### 12.3 Stress Tests

- deep nesting
- large slot lists

---

## Alignment Gaps (Current Code vs Target)

As of this revision, the most important gaps are:

1. **Explicit per-container overflow policy is missing in public container specs.**  
   Current behavior is mostly implicit clipping with optional warnings; there is
   no first-class `OverflowPolicy` field on all containers.

2. **Slot parameters are narrower than target.**  
   Current strict slots primarily expose main-axis fraction/fill and alignment.
   Slot-level margin and explicit cross-axis size modes are not first-class.

3. **Container primitive set is incomplete.**  
   `Stack`, `ScrollView`, and a formal `Wrap/Flow` primitive are not yet
   implemented as first-class declarative containers.

4. **Compression policy is not yet a documented, public contract.**  
   Existing distribution logic is deterministic, but the precise compress order
   and fallback behavior should be explicitly modeled and tested as API-level
   policy.

5. **Diagnostics output is limited.**  
   Validation failures are strong, but runtime layout diagnostics are not yet
   emitted as structured per-node results in release flows.

6. **Dirty-propagation and measure caching are not first-class in declarative
   layout.**  
   Current layout is deterministic and fast for present plugin sizes, but
   explicit subtree dirty tracking and cache keying are not yet exposed as a
   formal subsystem.

7. **Property/stress coverage can be expanded.**  
   Golden/unit coverage is strong, but dedicated property tests and large-scale
   stress tests for deep trees/large slot lists should be added to lock long-run
   behavior.

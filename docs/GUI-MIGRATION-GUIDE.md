# GUI Migration Guide (to Slot Grammar)

## Required End State
- Pure-data `UiSpec` each frame.
- `UiAction` reducer for all interactions.
- Container/slot/widget grammar enforced by validation.

## Breaking Changes
- Section naming replaced by slot naming:
  - `row_sections` -> `row_slots`
  - `column_sections` -> `column_slots`
  - `weighted` -> `weighted_slot`
  - `fraction` -> `fraction_slot`
  - `fill_section` -> `fill_slot`
  - `weighted_section_lengths` -> `weighted_slot_lengths`
- Section `Px` slot tracks are removed from slot helpers.
- Root content is now a required slot wrapper (applied automatically by constructors).
- Containers now require slot children (applied automatically by constructors).

## Migration Workflow
1. Replace old section helper imports with slot helpers.
2. Remove any px-based slot sizing and convert to fraction/fill.
3. Keep all composition inside declarative nodes and slot helpers.
4. Run `measure_checked` tests and fix structural errors first.
5. Re-run render/interaction tests for behavior parity.

## Typical Validation Failures
- `InvalidRootContent`: root content is not a slot.
- `InvalidRootSlotChild`: root slot child is not a container.
- `InvalidContainerChild`: container directly contains non-slot child.
- `InvalidSlotChild`: slot contains a slot or unsupported child.
- `InvalidSlotTrack` / `InvalidSlotFractions`: invalid slot track definitions.

## Recommended Rollout Order
1. Migrate root composition + top-level sections.
2. Migrate nested section helpers.
3. Migrate test tree matchers to account for slot wrappers.
4. Rebaseline screenshots/layout assertions if tree shape changed.

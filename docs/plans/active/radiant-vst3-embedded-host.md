# Radiant VST3 Embedded Host

## Objective

Make Toybox own reusable VST3 hosted-view lifecycle and use Radiant's embedded
Vello renderer for plugin GUIs, so plugins contain no native paint replay code.

## Scope

- Add Radiant's host-driven embedded Vello renderer as a pinned Toybox dependency.
- Add a reusable Toybox editor-runtime contract for Radiant paint plans and input.
- Move macOS VST3 `NSView` lifecycle, redraw scheduling, resize, and input forwarding
  out of Pump into Toybox.
- Keep Pump responsible only for DSP, state/parameters, and declarative UI composition.

## Definition of Done

- Toybox creates and owns the host-parented native view.
- Radiant Vello renders every `SurfacePaintPlan` for that view.
- Toybox forwards pointer, modifier, keyboard, resize, and redraw events generically.
- Pump has no AppKit primitive renderer or VST3 Cocoa view implementation.
- Focused hosted-view tests and the normal Toybox validation lane pass.

## Progress

- Radiant's host-driven embedded Vello renderer is pinned and consumed directly.
- The generic editor contract and macOS hosted view compile with focused tests passing.
- Toybox's focused tests, feature-specific clippy lane, and normal local CI pass.
- A macOS main-thread smoke host attaches, draws a gradient `FillPath` through
  embedded Vello, detaches, and exits cleanly.
- The smoke host asserts that the editor receives its logical size before the
  first paint plan is requested.
- Remaining: Pump migration and real-host VST3 verification.

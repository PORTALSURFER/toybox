# VST3 Guide

## Overview

Toybox exposes VST3 support behind the `vst3` feature flag:

- Framework module: `toybox::vst3`
- Convenience imports: `toybox::vst3::prelude::*`
- Entry macro: `toybox::vst3_plugin_entry!(FactoryType)`

The VST3 SDK source tree is tracked as a git submodule at:

- `third_party/vst3sdk`

## Prerequisites

1. Initialize submodules:
- `git submodule update --init --recursive`

2. Build with VST3 enabled:
- Add `features = ["vst3"]` to your `toybox` dependency.

Example dependency:

```toml
[dependencies]
toybox = { path = "../..", features = ["vst3"] }
```

## Authoring model

Use a shared plugin core and format-specific adapters:

- CLAP adapter via `toybox::clap`
- VST3 adapter via `toybox::vst3`

For VST3-specific wiring, implement Steinberg interfaces using types from
`toybox::vst3::prelude`.

## Key helper modules

- `toybox::vst3::bundle`:
  Windows `.vst3` bundle output helpers for `build.rs`.
- `toybox::vst3::component`:
  C-string and class metadata helpers for factory registration.
- `toybox::vst3::params`:
  Parameter normalization and process-block parameter queue iteration helpers.
- `toybox::vst3::processor`:
  Stereo f32 process buffer extraction helpers.
- `toybox::vst3::state`:
  Versioned payload serialization helpers on top of `IBStream`.
- `toybox::vst3::gui`:
  Platform/view helpers for host-parented plugin editors.

## Parented GUI bridge

When building with `features = ["vst3", "gui"]`, Toybox exposes:

- `parent_to_raw_window_handle(parent, platform)` in `toybox::vst3::gui`
- `Vst3HostedGui` + `HostedVst3View` in `toybox::vst3::gui`

This helper converts the host-provided VST3 `IPlugView::attached` parent
pointer and platform id into a `raw_window_handle::RawWindowHandle` suitable
for Patchbay GUI hosting.

Current support:

- Windows (`kPlatformTypeHWND`) is supported.
- Non-Windows currently returns `None`; plugins should fail attach or skip
  custom UI on unsupported targets.

## Reusable VST3 hosted view

For Patchbay-backed editors, use the shared hosted view lifecycle helper:

- Implement `Vst3HostedGui` for your plugin-specific GUI adapter.
- Return `HostedVst3View::new(adapter, width, height)` from `createView`.

This centralizes attach/open/remove/size logic so plugins do not duplicate
custom `IPlugViewTrait` boilerplate.

## Minimal example

Reference implementation:

- `examples/minimal-vst3`

It demonstrates:

- Audio processing (`IAudioProcessorTrait`)
- Parameter automation (`IParameterChanges`)
- State save/load (`IBStream`)
- Basic host-parented view (`IPlugViewTrait`)
- Factory export (`toybox::vst3_plugin_entry!`)

## Safety and constraints

- VST3 APIs are COM-style and contain unsafe calls.
- Keep all unsafe blocks narrow and justified.
- Avoid allocations in realtime process callbacks.
- Validate host pointers and handle null/invalid inputs defensively.

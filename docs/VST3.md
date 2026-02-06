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

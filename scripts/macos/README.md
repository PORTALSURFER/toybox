# Hosted editor regression checks

Use a Mac with a working Metal device and `VST3_SDK_DIR` pointing to the pinned
Steinberg SDK. The smoke example runs on the main thread and exercises normal
creation, rollback after an initialization panic, and quarantine after a native
redraw callback panic. Each invocation must run in a separate process:

```sh
cargo run --example radiant-vst3-host-smoke --features radiant-vst3
cargo run --example radiant-vst3-host-smoke --features radiant-vst3 -- resize
cargo run --example radiant-vst3-host-smoke --features radiant-vst3 -- redraw
```

The panic cases must exit successfully, print the injected panic and containment
diagnostic, remove all child views, and successfully open a fresh editor. The
example asserts that `RawWindowMetalLayer` was never registered. Run the VST3
unit tests too: `cargo test --features radiant-vst3` includes a test through the
generated COM ABI proving that an attachment panic reports failure and
quarantines the failed view.

For the independent-library regression, build Pump and GainSnap with matching
fixed Radiant/Toybox revisions, then run:

```sh
python3 scripts/macos/check-plugin-editors.py \
  /absolute/path/pump.vst3/Contents/MacOS/pump \
  /absolute/path/gainsnap.vst3/Contents/MacOS/gainsnap \
  --legacy-pump /absolute/path/old-pump.vst3/Contents/MacOS/pump \
  --legacy-gainsnap /absolute/path/old-gainsnap.vst3/Contents/MacOS/gainsnap \
  --output /absolute/path/results
```

The legacy arguments are optional controls for mixed-version compatibility.
The runner builds a small Objective-C++ VST3 host, records executable SHA-256 and
Mach-O UUIDs, and runs each ordering in a fresh subprocess with a timeout. Tests
cover both orders, repeated instances, resize, close/reopen, and closing the
first editor before opening the other. Fixed binaries also reject unclaimed
Space and character callbacks through the actual VST3 ABI. Fixed pairs must emit no duplicate ObjC
class warnings. Old binaries may retain warnings about their native fixtures.

This probe does not attach to the user's DAW or process audio. It requests
native display but does not certify visual or audible DAW acceptance. Validate
both editors together in a fresh Ableton session, including resizing, moving
between different-DPI displays, hide/show, parameter edits, and playback.

## Ownership and failure behavior

The shared macOS host owns a plain CAMetalLayer sublayer. Radiant receives an
explicit layer handle and bypasses the raw-window-metal observer-class path.
The host preserves its existing NSView backing layer and drawing callbacks,
updates sublayer geometry/scale, and destroys the renderer before the layer.
Partial attachment uses an RAII owner; native callbacks and VST3 callbacks catch
Rust unwinds before their non-unwinding ABIs. Failed editor state is quarantined
until host removal/recreation. The default panic hook remains intact so source
locations are retained. These boundaries cannot contain process aborts or
arbitrary Objective-C exceptions.

## Keyboard passthrough

VST3 returns the editor's handled result to the DAW. In callback-only mode,
AppKit events bypass plugin dispatch and continue to the host responder.
Native delivery forwards ignored events in the same way. Win32 handlers defer
unhandled keys and characters to their default processing. Editor adapters must
return actual consumption, using Radiant's `dispatch_keyboard_event` rather
than interpreting a routed widget ID as success. Built-in control keys, active
text fields, and explicit plugin shortcuts remain available.

# Embedded GPUI editors

Enable `toybox/gpui-gui` for CLAP editors or `toybox/gpui-vst3` for VST3.
Use `toybox::gpui` for GPUI types so the view and hosting platform always use
the same dependency revision. Toybox's default features stay empty; existing
Radiant and Patchbay consumers can continue enabling their explicit features.

## Ownership

Toybox owns the child NSView or HWND, renderer, text system, event translation,
UI dispatcher, and host-format adapters. A plugin supplies a GPUI root view
factory, parameter gesture logic, and visibility behavior. Keep plugin-specific
controls and DSP outside the backend.

`GpuiHostedGui::new(class_name, factory, width, height)` creates the facade.
The factory receives `&mut gpui::Window` and `&mut gpui::App` and returns an
`AnyView`. Configure the logical bounds with `with_size_contract(minimum,
preferred, maximum)` and optional visibility notifications with
`with_visibility_callback`.

The host supplies a native parent before opening. Keep the parent alive until
the facade closes. Open, resize, input, frame pumping, and close all belong on
the host UI thread. The host owns its event loop: never call GPUI's desktop
application runner from an audio plugin. No GUI operation belongs on the audio
processing thread.

Use `gpui_clap_gui_callbacks!` for the CLAP lifecycle and
`vst3::gui::create_gpui_view` for VST3. These adapters preserve normal host
parameter gestures and contain failures at the native ABI boundary.

## Input and visibility

Native text input and VST3 key callbacks feed the same GPUI view. A focused
control should consume only keys it handles; an unfocused editor must leave
transport keys available to the host. Text fields own selection, composition,
clipboard actions, and edit commit/cancel behavior. Do not recreate the input
entity during meter updates.

Treat visibility separately from focus. Hiding or closing a plugin can end a
plugin-specific listening mode, while changing focus must not do so. Invoke
parameter changes through the same automation path used by normal controls.

Nested native callbacks are serialized around GPUI updates. Closing makes
window state unavailable immediately, while retained callback ownership keeps
it alive until active callbacks return. VST3 removal that re-enters another
GUI callback is deferred and drained once the outer callback releases its
facade lock.

## Validation

Run `cargo test -p toybox --features gpui-gui` for portable backend tests and
`cargo test -p toybox --features gpui-vst3` with an initialized `VST3_SDK_DIR`
for format integration. Native AppKit smoke programs must execute on the
process main thread, not a Rust test worker thread. Windows native behavior
requires the Windows CI job or a Windows host.

`capture_rgba()` renders the actual current GPUI scene into packed RGBA pixels
and returns its physical width and height. It is a synchronous GPU readback
for visual tests, not a realtime metering API. A screenshot runner should own
its native parent fixture, open the normal editor, capture, and close before
releasing the parent. Retina captures can be resampled to the desired logical
image dimensions for release screenshots.

Before shipping a new backend revision, test multiple editor instances and
two independently linked plugin libraries in both load orders. Also exercise
resize, hide/minimize, close/reopen, focused text and arrows, and unhandled
transport keys in a real host. Automated rendering and controller tests do
not replace a fresh DAW reload and user audition.

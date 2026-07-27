//! Host-neutral Radiant editor hosting for embedded plugin views.
//!
//! The bridge deliberately has no dependency on the legacy Patchbay `gui`
//! feature. Plugins provide a retained Radiant editor and use the same facade
//! from CLAP or VST3 host callbacks.

use radiant::runtime::{Event, SurfacePaintPlan};
use radiant::widgets::WidgetKey;
use raw_window_handle::RawWindowHandle;

/// Editor implementation consumed by [`RadiantHostedGui`].
pub trait RadiantEditor: 'static {
    /// Resize the editor's logical viewport.
    fn resize(&mut self, width: u32, height: u32);

    /// Dispatch one backend-neutral Radiant event.
    fn dispatch_event(&mut self, event: Event);

    /// Return the latest retained paint plan.
    fn paint_plan(&mut self) -> &SurfacePaintPlan;

    /// Report whether animation requires periodic redraws.
    fn needs_realtime_redraw(&self) -> bool;

    /// Dispatch a semantic key press.
    fn dispatch_key_press(&mut self, key: WidgetKey) -> bool;

    /// Dispatch one text character.
    fn dispatch_character(&mut self, character: char) -> bool;

    /// Cancel an active text or numeric entry.
    fn cancel_text_entry(&mut self) -> bool;
}

pub(crate) fn vst3_key_down_to_input_char(key: u16, key_code: i16) -> Option<char> {
    match key_code {
        8 => Some('\u{8}'),
        9 => Some('\t'),
        13 => Some('\r'),
        27 => Some('\u{1b}'),
        32 => Some(' '),
        37 => Some('\u{1c}'),
        38 => Some('\u{1e}'),
        39 => Some('\u{1d}'),
        40 => Some('\u{1f}'),
        46 => Some('\u{7f}'),
        _ => char::from_u32(key as u32).or_else(|| {
            (0x20..=0x7e)
                .contains(&(key_code as u32))
                .then_some(key_code as u8 as char)
        }),
    }
}
/// Compatibility trait name retained for existing Radiant VST3 callers.
pub use RadiantEditor as RadiantVst3Editor;

/// Platform host operations shared by the CLAP and VST3 facades.
pub(crate) trait HostedGui {
    fn set_parent_raw(&mut self, parent: RawWindowHandle);
    fn open(&mut self) -> bool;
    fn close(&mut self);
    fn last_size(&self) -> Option<(u32, u32)>;
    fn request_resize(&self, width: u32, height: u32);
    fn on_key_down(&self, key: u16, key_code: i16, modifiers: i16) -> bool;
    fn on_key_up(&self, key: u16, key_code: i16, modifiers: i16) -> bool;
}
pub(crate) use HostedGui as Vst3HostedGui;

/// Inclusive logical size contract shared by embedded Radiant hosts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorSizeContract {
    /// Default logical editor size.
    pub default: (u32, u32),
    /// Minimum supported logical editor size.
    pub minimum: (u32, u32),
    /// Maximum supported logical editor size.
    pub maximum: (u32, u32),
}

impl EditorSizeContract {
    /// Construct an ordered contract, retaining the default inside its bounds.
    pub fn new(default: (u32, u32), minimum: (u32, u32), maximum: (u32, u32)) -> Self {
        let default = (default.0.max(1), default.1.max(1));
        let minimum = (
            minimum.0.max(1).min(default.0),
            minimum.1.max(1).min(default.1),
        );
        let maximum = (
            maximum.0.max(default.0).max(minimum.0),
            maximum.1.max(default.1).max(minimum.1),
        );
        Self {
            default,
            minimum,
            maximum,
        }
    }

    /// Clamp a host request while preserving the contract's default aspect ratio.
    pub fn constrain(&self, requested: (u32, u32)) -> (u32, u32) {
        let default_width = self.default.0 as f64;
        let default_height = self.default.1 as f64;
        let min_scale =
            (self.minimum.0 as f64 / default_width).max(self.minimum.1 as f64 / default_height);
        let max_scale =
            (self.maximum.0 as f64 / default_width).min(self.maximum.1 as f64 / default_height);
        let requested_scale = (requested.0.max(1) as f64 / default_width)
            .min(requested.1.max(1) as f64 / default_height);
        let scale = requested_scale.clamp(min_scale, max_scale);
        let width = (default_width * scale).round() as u32;
        let height = (default_height * scale).round() as u32;
        (
            width.clamp(self.minimum.0, self.maximum.0).max(1),
            height.clamp(self.minimum.1, self.maximum.1).max(1),
        )
    }
}

#[cfg(target_os = "macos")]
#[path = "../vst3/gui/radiant_host_macos.rs"]
mod host_macos;

#[cfg(target_os = "macos")]
use host_macos::RadiantVst3HostedGui as PlatformHostedGui;

struct EditorAdapter(Box<dyn RadiantEditor>);

impl host_macos::RadiantVst3Editor for EditorAdapter {
    fn resize(&mut self, width: u32, height: u32) {
        self.0.resize(width, height);
    }

    fn dispatch_event(&mut self, event: Event) {
        self.0.dispatch_event(event);
    }

    fn paint_plan(&mut self) -> &SurfacePaintPlan {
        self.0.paint_plan()
    }

    fn needs_realtime_redraw(&self) -> bool {
        self.0.needs_realtime_redraw()
    }

    fn dispatch_key_press(&mut self, key: WidgetKey) -> bool {
        self.0.dispatch_key_press(key)
    }

    fn dispatch_character(&mut self, character: char) -> bool {
        self.0.dispatch_character(character)
    }

    fn cancel_text_entry(&mut self) -> bool {
        self.0.cancel_text_entry()
    }
}

/// CLAP/VST3-compatible host facade for a retained Radiant editor.
pub struct RadiantHostedGui {
    inner: PlatformHostedGui,
    contract: EditorSizeContract,
}

/// Compatibility alias for callers that still name the VST3 host explicitly.
pub type RadiantVst3HostedGui = RadiantHostedGui;

impl RadiantHostedGui {
    /// Create a host facade with an explicit default logical size.
    pub fn new(
        class_name: &'static str,
        editor: impl RadiantEditor,
        width: u32,
        height: u32,
    ) -> Self {
        let contract = EditorSizeContract::new((width, height), (1, 1), (u32::MAX, u32::MAX));
        #[cfg(target_os = "macos")]
        let platform_editor = EditorAdapter(Box::new(editor));
        #[cfg(not(target_os = "macos"))]
        let platform_editor = editor;
        Self {
            inner: PlatformHostedGui::new(class_name, platform_editor, width, height),
            contract,
        }
    }

    /// Construct a Win32 host whose editor is created when the child opens.
    /// Declare minimum, default, and maximum logical sizes for the editor.
    pub fn with_size_contract(
        mut self,
        minimum: (u32, u32),
        default: (u32, u32),
        maximum: (u32, u32),
    ) -> Self {
        self.contract = EditorSizeContract::new(default, minimum, maximum);
        let (width, height) = self.contract.default;
        <PlatformHostedGui as Vst3HostedGui>::request_resize(&self.inner, width, height);
        self
    }

    /// Configure embedded font options for the AppKit renderer.
    #[cfg(target_os = "macos")]
    pub fn with_text_options(mut self, options: radiant::runtime::NativeTextOptions) -> Self {
        self.inner = self.inner.with_text_options(options);
        self
    }

    /// Return the shared logical size contract.
    pub fn size_contract(&self) -> EditorSizeContract {
        self.contract
    }

    /// Attach a host-provided parent window.
    pub fn set_parent(&mut self, parent: RawWindowHandle) {
        <PlatformHostedGui as Vst3HostedGui>::set_parent_raw(&mut self.inner, parent);
    }

    /// Attach a CLAP host-provided parent window.
    pub fn set_parent_window(&mut self, window: clack_extensions::gui::Window<'_>) {
        use raw_window_handle::HasRawWindowHandle;
        self.set_parent(window.raw_window_handle());
    }

    /// Constrain one host resize request using the shared editor contract.
    pub fn constrain_size(&self, size: (u32, u32)) -> (u32, u32) {
        self.contract.constrain(size)
    }

    /// Create and attach the retained native child view.
    pub fn open(&mut self) -> bool {
        let (width, height) = self.contract.constrain(self.contract.default);
        <PlatformHostedGui as Vst3HostedGui>::request_resize(&self.inner, width, height);
        <PlatformHostedGui as Vst3HostedGui>::open(&mut self.inner)
    }

    /// Show an already-created child view.
    pub fn show(&mut self) {
        let _ = self.open();
        self.inner.show();
    }

    /// Hide an already-created child view without destroying its editor state.
    pub fn hide(&mut self) {
        self.inner.hide();
    }

    /// Destroy the native child view while retaining the editor for a later open.
    pub fn close(&mut self) {
        <PlatformHostedGui as Vst3HostedGui>::close(&mut self.inner);
    }

    /// Return the most recent logical size.
    pub fn last_size(&self) -> Option<(u32, u32)> {
        <PlatformHostedGui as Vst3HostedGui>::last_size(&self.inner)
    }

    /// Apply a constrained host resize without host callback feedback.
    pub fn request_resize(&self, width: u32, height: u32) {
        let (width, height) = self.contract.constrain((width, height));
        <PlatformHostedGui as Vst3HostedGui>::request_resize(&self.inner, width, height);
    }

    /// Apply host DPI scale; platform renderers also refresh backing scale on draw.
    pub fn set_scale(&self, scale: f64) {
        self.inner.set_scale(scale);
    }

    /// Forward one semantic key press from a VST3 host.
    pub fn on_key_down(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        <PlatformHostedGui as Vst3HostedGui>::on_key_down(&self.inner, key, key_code, modifiers)
    }

    /// Forward one semantic key release from a VST3 host.
    pub fn on_key_up(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        <PlatformHostedGui as Vst3HostedGui>::on_key_up(&self.inner, key, key_code, modifiers)
    }
}

/// Inject the standard CLAP GUI lifecycle callbacks for a [`RadiantHostedGui`].
///
/// Only Cocoa and Win32 are advertised. Linux intentionally returns `false`
/// until a native child host is available rather than silently falling back to
/// an unsupported X11 surface.
#[macro_export]
macro_rules! radiant_clap_gui_callbacks {
    (gui = $gui:ident, preferred_size = $preferred:path, show = $show:expr) => {
        fn is_api_supported(
            &mut self,
            configuration: $crate::clack_extensions::gui::GuiConfiguration,
        ) -> bool {
            if !cfg!(target_os = "macos") {
                return false;
            }
            let Some(api_type) =
                $crate::clack_extensions::gui::GuiApiType::default_for_current_platform()
            else {
                return false;
            };
            configuration.api_type == api_type && !configuration.is_floating
        }

        fn get_preferred_api(
            &'_ mut self,
        ) -> Option<$crate::clack_extensions::gui::GuiConfiguration<'_>> {
            if !cfg!(target_os = "macos") {
                return None;
            }
            let api_type =
                $crate::clack_extensions::gui::GuiApiType::default_for_current_platform()?;
            Some($crate::clack_extensions::gui::GuiConfiguration {
                api_type,
                is_floating: false,
            })
        }

        fn create(
            &mut self,
            _configuration: $crate::clack_extensions::gui::GuiConfiguration,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            Ok(())
        }

        fn destroy(&mut self) {
            self.$gui.close();
        }

        fn set_scale(
            &mut self,
            scale: f64,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$gui.set_scale(scale);
            Ok(())
        }

        fn get_size(&mut self) -> Option<$crate::clack_extensions::gui::GuiSize> {
            let (width, height) = self.$gui.last_size().unwrap_or_else($preferred);
            Some($crate::clack_extensions::gui::GuiSize { width, height })
        }

        fn can_resize(&mut self) -> bool {
            true
        }

        fn adjust_size(
            &mut self,
            size: $crate::clack_extensions::gui::GuiSize,
        ) -> Option<$crate::clack_extensions::gui::GuiSize> {
            let (width, height) = self.$gui.constrain_size((size.width, size.height));
            Some($crate::clack_extensions::gui::GuiSize { width, height })
        }

        fn set_size(
            &mut self,
            size: $crate::clack_extensions::gui::GuiSize,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$gui.request_resize(size.width, size.height);
            Ok(())
        }

        fn set_parent(
            &mut self,
            window: $crate::clack_extensions::gui::Window<'_>,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$gui.set_parent_window(window);
            Ok(())
        }

        fn set_transient(
            &mut self,
            _window: $crate::clack_extensions::gui::Window<'_>,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            Ok(())
        }

        fn show(&mut self) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$gui.show();
            ($show)(self)
        }

        fn hide(&mut self) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$gui.hide();
            Ok(())
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{EditorSizeContract, RadiantEditor, RadiantHostedGui};
    use radiant::runtime::{Event, SurfacePaintPlan};
    use radiant::widgets::WidgetKey;

    #[test]
    fn size_contract_orders_bounds_and_preserves_default() {
        let contract = EditorSizeContract::new((912, 684), (720, 540), (1440, 1080));
        assert_eq!(contract.constrain((1, 1)), (720, 540));
        assert_eq!(contract.constrain((912, 684)), (912, 684));
        assert_eq!(contract.constrain((5000, 5000)), (1440, 1080));
    }

    #[test]
    fn size_contract_preserves_aspect_with_asymmetric_bounds() {
        let contract = EditorSizeContract::new((1000, 500), (500, 400), (2000, 1200));
        assert_eq!(contract.constrain((1, 1)), (800, 400));
        assert_eq!(contract.constrain((2000, 400)), (800, 400));
        assert_eq!(contract.constrain((2000, 1000)), (2000, 1000));
        assert_eq!(contract.constrain((5000, 5000)), (2000, 1000));
    }

    #[test]
    fn size_contract_updates_preopen_host_size() {
        let gui = RadiantHostedGui::new("ToyboxRadiantPreopenContractTest", MockEditor, 420, 282)
            .with_size_contract((720, 540), (912, 684), (1440, 1080));
        assert_eq!(gui.last_size(), Some((912, 684)));
    }

    struct MockEditor;

    impl RadiantEditor for MockEditor {
        fn resize(&mut self, _width: u32, _height: u32) {}
        fn dispatch_event(&mut self, _event: Event) {}
        fn paint_plan(&mut self) -> &SurfacePaintPlan {
            unreachable!("paint plan is not used by contract tests")
        }
        fn needs_realtime_redraw(&self) -> bool {
            false
        }
        fn dispatch_key_press(&mut self, _key: WidgetKey) -> bool {
            false
        }
        fn dispatch_character(&mut self, _character: char) -> bool {
            false
        }
        fn cancel_text_entry(&mut self) -> bool {
            false
        }
    }
}

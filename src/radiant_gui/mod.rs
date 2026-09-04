//! Host-neutral Radiant editor hosting for embedded plugin views.
//!
//! The bridge deliberately has no dependency on the legacy Patchbay `gui`
//! feature. Plugins provide a retained Radiant editor and use the same facade
//! from CLAP or VST3 host callbacks.

use radiant::runtime::{Event, SurfacePaintPlan};
#[cfg(target_os = "windows")]
use radiant::theme::DpiScale;
use radiant::widgets::{KeyboardModifiers, PointerModifiers, WidgetKey};
use raw_window_handle::RawWindowHandle;

/// Shared bundled-font options for Radiant native text.
mod typography;

pub use typography::{bundled_offscreen_capture, bundled_text_options};

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
    fn dispatch_key_press(&mut self, key: WidgetKey, modifiers: KeyboardModifiers) -> bool;

    /// Dispatch one text character.
    fn dispatch_character(&mut self, character: char) -> bool;

    /// Dispatch one command-modified textual shortcut.
    ///
    /// Returning `true` consumes the shortcut. The default keeps existing
    /// editors on the host responder-chain path.
    fn dispatch_shortcut(&mut self, _character: char, _modifiers: PointerModifiers) -> bool {
        false
    }

    /// Cancel an active text or numeric entry.
    fn cancel_text_entry(&mut self) -> bool;
}

/// Convert a VST3 key callback into the character understood by Radiant.
pub(crate) fn vst3_key_down_to_input_char(key: u16, key_code: i16) -> Option<char> {
    match key_code {
        // VST3's `VirtualKeyCodes` are deliberately not Win32 virtual-key
        // values. Keep this fallback dependency-free so the Radiant CLAP
        // build can still compile the Windows host without enabling VST3.
        1 => Some('\u{8}'),     // KEY_BACK
        2 => Some('\t'),        // KEY_TAB
        4 | 19 => Some('\r'),   // KEY_RETURN / KEY_ENTER
        6 => Some('\u{1b}'),    // KEY_ESCAPE
        7 => Some(' '),         // KEY_SPACE
        9 => Some('\u{f72b}'),  // KEY_END
        10 => Some('\u{f729}'), // KEY_HOME
        11 => Some('\u{1c}'),   // KEY_LEFT
        12 => Some('\u{1e}'),   // KEY_UP
        13 => Some('\u{1d}'),   // KEY_RIGHT
        14 => Some('\u{1f}'),   // KEY_DOWN
        22 => Some('\u{7f}'),   // KEY_DELETE
        _ => char::from_u32(key as u32).or_else(|| {
            (0x20..=0x7e)
                .contains(&(key_code as u32))
                .then_some(key_code as u8 as char)
        }),
    }
}

/// Convert logical editor dimensions to the physical pixels required by a
/// native host window.
#[cfg(target_os = "windows")]
pub(crate) fn logical_size_to_physical(
    logical_width: u32,
    logical_height: u32,
    dpi_scale: DpiScale,
) -> (u32, u32) {
    fn dimension(value: f32) -> u32 {
        if !value.is_finite() {
            return 1;
        }
        value.ceil().clamp(1.0, u32::MAX as f32) as u32
    }

    (
        dimension(dpi_scale.logical_to_physical(logical_width.max(1) as f32)),
        dimension(dpi_scale.logical_to_physical(logical_height.max(1) as f32)),
    )
}

/// Convert physical host pixels to integer logical editor dimensions.
#[cfg(target_os = "windows")]
pub(crate) fn physical_size_to_logical(
    physical_width: u32,
    physical_height: u32,
    dpi_scale: DpiScale,
) -> (u32, u32) {
    fn dimension(value: f32) -> u32 {
        if !value.is_finite() {
            return 1;
        }
        value.floor().clamp(1.0, u32::MAX as f32) as u32
    }

    (
        dimension(dpi_scale.physical_to_logical(physical_width.max(1) as f32)),
        dimension(dpi_scale.physical_to_logical(physical_height.max(1) as f32)),
    )
}
/// Compatibility trait name retained for existing Radiant VST3 callers.
pub use RadiantEditor as RadiantVst3Editor;

/// Platform host operations shared by the CLAP and VST3 facades.
pub(crate) trait HostedGui {
    /// Attach the host-owned parent window.
    fn set_parent_raw(&mut self, parent: RawWindowHandle);
    /// Create and attach the native child view.
    fn open(&mut self) -> bool;
    /// Remove the native child view while retaining host state.
    fn close(&mut self);
    /// Return the latest negotiated logical size.
    fn last_size(&self) -> Option<(u32, u32)>;
    /// Show the already-open native child view.
    fn show(&self) -> bool;
    /// Replace the default logical size used before the child opens.
    fn set_default_size(&mut self, width: u32, height: u32);
    /// Select callback-only keyboard delivery for VST3, or native delivery for CLAP.
    fn set_callback_keyboard_mode(&mut self, _callback_only: bool) {}
    /// Convert logical dimensions to the host-facing dimensions.
    fn host_size_from_logical(&self, width: u32, height: u32) -> (u32, u32);
    /// Convert host-facing dimensions to logical editor dimensions.
    fn logical_size_from_host(&self, width: u32, height: u32) -> (u32, u32);
    /// Request a host-facing resize from the native child view.
    fn request_resize(&self, width: u32, height: u32);
    /// Dispatch one host key-down callback.
    fn on_key_down(&self, key: u16, key_code: i16, modifiers: i16) -> bool;
    /// Dispatch one host key-up callback.
    fn on_key_up(&self, key: u16, key_code: i16, modifiers: i16) -> bool;
    /// Apply focus requested by the host to the native child view.
    fn on_focus(&self, focused: bool) -> bool;
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

#[cfg(target_os = "windows")]
#[path = "../vst3/gui/radiant_host_windows.rs"]
mod host_windows;

#[cfg(target_os = "windows")]
use host_windows::RadiantWindowsHostedGui as PlatformHostedGui;

/// Adapt the public host-neutral editor trait to the macOS VST3 bridge.
#[cfg(target_os = "macos")]
struct EditorAdapter(Box<dyn RadiantEditor>);

#[cfg(target_os = "macos")]
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

    fn dispatch_key_press(&mut self, key: WidgetKey, modifiers: KeyboardModifiers) -> bool {
        self.0.dispatch_key_press(key, modifiers)
    }

    fn dispatch_character(&mut self, character: char) -> bool {
        self.0.dispatch_character(character)
    }

    fn dispatch_shortcut(&mut self, character: char, modifiers: PointerModifiers) -> bool {
        self.0.dispatch_shortcut(character, modifiers)
    }

    fn cancel_text_entry(&mut self) -> bool {
        self.0.cancel_text_entry()
    }
}

/// CLAP/VST3-compatible host facade for a retained Radiant editor.
pub struct RadiantHostedGui {
    /// Platform-specific hosted view implementation.
    inner: PlatformHostedGui,
    /// Logical size contract applied to host resize requests.
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
        <PlatformHostedGui as Vst3HostedGui>::set_default_size(&mut self.inner, width, height);
        self
    }

    /// Configure embedded font options for the native renderer.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn with_text_options(mut self, options: radiant::runtime::NativeTextOptions) -> Self {
        #[cfg(target_os = "macos")]
        {
            self.inner = self.inner.with_text_options(options);
        }
        #[cfg(target_os = "windows")]
        {
            self.inner = self.inner.with_text_options(options);
        }
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

    /// Constrain one host-facing resize request using the shared editor contract.
    pub fn constrain_size(&self, size: (u32, u32)) -> (u32, u32) {
        let logical = <PlatformHostedGui as Vst3HostedGui>::logical_size_from_host(
            &self.inner,
            size.0,
            size.1,
        );
        let logical = self.contract.constrain(logical);
        <PlatformHostedGui as Vst3HostedGui>::host_size_from_logical(
            &self.inner,
            logical.0,
            logical.1,
        )
    }

    /// Create and attach the retained native child view.
    pub fn open(&mut self) -> bool {
        let requested = self.last_size().unwrap_or(self.contract.default);
        let (width, height) = self.constrain_size(requested);
        if Some((width, height)) != self.last_size() {
            <PlatformHostedGui as Vst3HostedGui>::request_resize(&self.inner, width, height);
        }
        <PlatformHostedGui as Vst3HostedGui>::open(&mut self.inner)
    }

    /// Show a child that has already been opened by the lifecycle owner.
    fn show_open(&self) -> bool {
        <PlatformHostedGui as Vst3HostedGui>::show(&self.inner)
    }

    /// Show an already-created child view.
    pub fn show(&mut self) -> bool {
        if !self.open() {
            return false;
        }
        self.show_open()
    }

    /// Hide an already-created child view without destroying its editor state.
    pub fn hide(&mut self) {
        self.inner.hide();
    }

    /// Destroy the native child view while retaining the editor for a later open.
    pub fn close(&mut self) {
        <PlatformHostedGui as Vst3HostedGui>::close(&mut self.inner);
    }

    /// Return the most recent host-facing size.
    pub fn last_size(&self) -> Option<(u32, u32)> {
        <PlatformHostedGui as Vst3HostedGui>::last_size(&self.inner)
    }

    /// Apply a constrained host resize without host callback feedback.
    pub fn request_resize(&self, width: u32, height: u32) {
        let (width, height) = self.constrain_size((width, height));
        <PlatformHostedGui as Vst3HostedGui>::request_resize(&self.inner, width, height);
    }

    /// Apply host DPI scale; platform renderers also refresh backing scale on draw.
    pub fn set_scale(&self, scale: f64) {
        self.inner.set_scale(scale);
    }

    /// Select callback-only keyboard delivery for VST3 host callbacks.
    pub fn set_callback_keyboard_mode(&mut self, callback_only: bool) {
        <PlatformHostedGui as Vst3HostedGui>::set_callback_keyboard_mode(
            &mut self.inner,
            callback_only,
        );
    }

    /// Forward one semantic key press from a VST3 host.
    pub fn on_key_down(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        <PlatformHostedGui as Vst3HostedGui>::on_key_down(&self.inner, key, key_code, modifiers)
    }

    /// Forward one semantic key release from a VST3 host.
    pub fn on_key_up(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        <PlatformHostedGui as Vst3HostedGui>::on_key_up(&self.inner, key, key_code, modifiers)
    }

    /// Forward a host focus change to the native child view.
    pub fn on_focus(&self, focused: bool) -> bool {
        <PlatformHostedGui as Vst3HostedGui>::on_focus(&self.inner, focused)
    }
}

/// Bridge the Radiant facade into the public VST3 host trait without creating
/// a second lifecycle owner around the same native view.
#[cfg(all(
    feature = "radiant-vst3",
    any(target_os = "macos", target_os = "windows")
))]
impl crate::vst3::gui::Vst3HostedGui for RadiantHostedGui {
    fn set_parent_raw(&mut self, parent: RawWindowHandle) {
        RadiantHostedGui::set_parent(self, parent);
    }

    fn open(&mut self) -> bool {
        RadiantHostedGui::open(self)
    }

    fn close(&mut self) {
        RadiantHostedGui::close(self);
    }

    fn last_size(&self) -> Option<(u32, u32)> {
        RadiantHostedGui::last_size(self)
    }

    fn show(&self) -> bool {
        RadiantHostedGui::show_open(self)
    }

    fn set_callback_keyboard_mode(&mut self, callback_only: bool) {
        RadiantHostedGui::set_callback_keyboard_mode(self, callback_only);
    }

    fn request_resize(&self, width: u32, height: u32) {
        RadiantHostedGui::request_resize(self, width, height);
    }

    fn host_size_from_logical(&self, width: u32, height: u32) -> (u32, u32) {
        <PlatformHostedGui as Vst3HostedGui>::host_size_from_logical(&self.inner, width, height)
    }

    fn logical_size_from_host(&self, width: u32, height: u32) -> (u32, u32) {
        <PlatformHostedGui as Vst3HostedGui>::logical_size_from_host(&self.inner, width, height)
    }

    fn on_key_down(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        RadiantHostedGui::on_key_down(self, key, key_code, modifiers)
    }

    fn on_key_up(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        RadiantHostedGui::on_key_up(self, key, key_code, modifiers)
    }

    fn on_focus(&self, focused: bool) -> bool {
        RadiantHostedGui::on_focus(self, focused)
    }
}

#[cfg(all(
    test,
    feature = "radiant-vst3",
    any(target_os = "macos", target_os = "windows")
))]
mod vst3_trait_contract_tests {
    use super::RadiantVst3HostedGui;
    use crate::vst3::gui::{HostedVst3View, Vst3HostedGui};

    fn assert_host_trait<T: Vst3HostedGui>() {}
    fn assert_view_type<T: Vst3HostedGui>() {
        let _ = std::marker::PhantomData::<HostedVst3View<T>>;
    }

    #[test]
    fn radiant_hosted_gui_satisfies_public_vst3_view_contract() {
        assert_host_trait::<RadiantVst3HostedGui>();
        assert_view_type::<RadiantVst3HostedGui>();
    }
}

/// Inject the standard CLAP GUI lifecycle callbacks for a [`RadiantHostedGui`].
///
/// Only the native macOS or Windows embedded API is advertised; unsupported
/// platforms return `false`.
#[macro_export]
macro_rules! radiant_clap_gui_callbacks {
    (gui = $gui:ident, preferred_size = $preferred:path, show = $show:expr) => {
        fn is_api_supported(
            &mut self,
            configuration: $crate::clack_extensions::gui::GuiConfiguration,
        ) -> bool {
            if !cfg!(any(target_os = "macos", target_os = "windows")) {
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
            if !cfg!(any(target_os = "macos", target_os = "windows")) {
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
            self.$gui.set_callback_keyboard_mode(false);
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
            if !self.$gui.show() {
                return Err($crate::clack_plugin::plugin::PluginError::Message(
                    "Radiant editor could not open its host parent",
                ));
            }
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
    use super::{EditorSizeContract, RadiantEditor, RadiantHostedGui, vst3_key_down_to_input_char};
    use radiant::runtime::{Event, SurfacePaintPlan};
    use radiant::widgets::{KeyboardModifiers, PointerModifiers, WidgetKey};

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

    #[cfg(target_os = "windows")]
    #[test]
    fn physical_size_helpers_apply_dpi_once_and_round_renderer_bounds_up() {
        use super::{logical_size_to_physical, physical_size_to_logical};
        use radiant::theme::DpiScale;

        let scale = DpiScale::new(1.5);
        assert_eq!(logical_size_to_physical(912, 684, scale), (1368, 1026));
        assert_eq!(physical_size_to_logical(1368, 1026, scale), (912, 684));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn physical_size_helpers_keep_fractional_pixels_inside_the_host_surface() {
        use super::{logical_size_to_physical, physical_size_to_logical};
        use radiant::theme::DpiScale;

        let scale = DpiScale::new(1.5);
        assert_eq!(logical_size_to_physical(911, 683, scale), (1367, 1025));
        assert_eq!(physical_size_to_logical(1367, 1025, scale), (911, 683));
    }

    #[test]
    fn vst3_callback_translation_uses_vst3_virtual_key_codes() {
        assert_eq!(vst3_key_down_to_input_char(0, 1), Some('\u{8}'));
        assert_eq!(vst3_key_down_to_input_char(0, 11), Some('\u{1c}'));
        assert_eq!(vst3_key_down_to_input_char(0, 12), Some('\u{1e}'));
        assert_eq!(vst3_key_down_to_input_char(0, 13), Some('\u{1d}'));
        assert_eq!(vst3_key_down_to_input_char(0, 14), Some('\u{1f}'));
        assert_eq!(vst3_key_down_to_input_char(0, 10), Some('\u{f729}'));
        assert_eq!(vst3_key_down_to_input_char(0, 9), Some('\u{f72b}'));
        assert_eq!(vst3_key_down_to_input_char(0, 22), Some('\u{7f}'));
    }

    #[test]
    fn size_contract_updates_preopen_host_size() {
        let gui = RadiantHostedGui::new("ToyboxRadiantPreopenContractTest", MockEditor, 420, 282)
            .with_size_contract((720, 540), (912, 684), (1440, 1080));
        assert_eq!(gui.last_size(), Some((912, 684)));
    }

    #[test]
    fn show_reports_failure_when_host_parent_is_missing() {
        let mut gui = RadiantHostedGui::new("ToyboxRadiantShowFailureTest", MockEditor, 420, 282);
        assert!(!gui.show());
    }

    #[test]
    fn focus_reports_failure_before_a_native_child_is_open() {
        let gui = RadiantHostedGui::new("ToyboxRadiantFocusFailureTest", MockEditor, 420, 282);
        assert!(!gui.on_focus(true));
    }

    #[test]
    fn hide_show_preserves_negotiated_size() {
        let mut gui = RadiantHostedGui::new("ToyboxRadiantRetainedSizeTest", MockEditor, 420, 282)
            .with_size_contract((720, 540), (912, 684), (1440, 1080));
        gui.request_resize(1440, 1080);
        assert_eq!(gui.last_size(), Some((1440, 1080)));

        gui.hide();
        assert!(!gui.show(), "show without a host parent should fail");
        assert_eq!(gui.last_size(), Some((1440, 1080)));
    }

    #[test]
    fn legacy_editor_uses_unclaimed_shortcut_default() {
        let mut editor = MockEditor;
        assert!(!editor.dispatch_shortcut('z', PointerModifiers::default()));
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
        fn dispatch_key_press(&mut self, _key: WidgetKey, _modifiers: KeyboardModifiers) -> bool {
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

// Macro helpers that wire standard CLAP GUI callbacks to Patchbay.

/// Inject default CLAP resize callbacks backed by Toybox Patchbay policy.
///
/// This macro standardizes host-resize behavior across plugins: resizable by
/// default, with an opt-out via [`GuiHostWindow::set_host_resize_policy`].
#[cfg(feature = "gui")]
#[macro_export]
macro_rules! patchbay_clap_resize_callbacks {
    ($field:ident) => {
        fn can_resize(&mut self) -> bool {
            self.$field.host_resize_enabled()
        }

        fn adjust_size(
            &mut self,
            size: $crate::clack_extensions::gui::GuiSize,
        ) -> Option<$crate::clack_extensions::gui::GuiSize> {
            self.$field.adjust_host_size(size)
        }

        fn set_size(
            &mut self,
            size: $crate::clack_extensions::gui::GuiSize,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$field.apply_host_size(size);
            Ok(())
        }
    };
}

/// Inject a full default CLAP GUI callback implementation for Patchbay windows.
///
/// This macro owns all host-facing GUI callback plumbing so plugins can focus
/// on UI state/build/reducer logic. The plugin still supplies:
/// - a GUI wrapper field identifier (`gui = ...`)
/// - a preferred-size function path (`preferred_size = ...`)
/// - a `show` callback expression (`show = ...`) that opens the GUI state
///
/// Resize policy remains Toybox-owned via `GuiHostWindow` defaults, with
/// opt-out available through `GuiHostWindow::set_host_resize_policy`.
#[cfg(feature = "gui")]
#[macro_export]
macro_rules! patchbay_clap_gui_callbacks {
    (
        gui = $gui:ident,
        preferred_size = $preferred:path,
        show = $show:expr
    ) => {
        fn is_api_supported(
            &mut self,
            configuration: $crate::clack_extensions::gui::GuiConfiguration,
        ) -> bool {
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
            let Some(api_type) =
                $crate::clack_extensions::gui::GuiApiType::default_for_current_platform()
            else {
                return None;
            };

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
            _scale: f64,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            Ok(())
        }

        fn get_size(&mut self) -> Option<$crate::clack_extensions::gui::GuiSize> {
            if let Some((width, height)) = self.$gui.last_size() {
                return Some($crate::clack_extensions::gui::GuiSize { width, height });
            }
            let (width, height) = $preferred();
            Some($crate::clack_extensions::gui::GuiSize { width, height })
        }

        $crate::patchbay_clap_resize_callbacks!($gui);

        fn set_parent(
            &mut self,
            window: $crate::clack_extensions::gui::Window<'_>,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$gui.set_parent(window);
            Ok(())
        }

        fn set_transient(
            &mut self,
            _window: $crate::clack_extensions::gui::Window<'_>,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            Ok(())
        }

        fn show(&mut self) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            ($show)(self)
        }

        fn hide(&mut self) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$gui.close();
            Ok(())
        }
    };
}

/// Inject the standard CLAP GUI callback implementation for an embedded GPUI
/// host. The editor supplies its factory and size contract; Toybox owns the
/// parent-window lifecycle and visibility transitions.
#[cfg(feature = "gpui-gui")]
#[macro_export]
macro_rules! gpui_clap_gui_callbacks {
    (
        gui = $gui:ident,
        preferred_size = $preferred:path,
        show = $show:expr
    ) => {
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
            _scale: f64,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
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
            Some(size)
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
            self.$gui.set_parent(window);
            Ok(())
        }

        fn set_transient(
            &mut self,
            _window: $crate::clack_extensions::gui::Window<'_>,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            Ok(())
        }

        fn show(&mut self) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            if !self.$gui.open() || !self.$gui.show() {
                Err($crate::clack_plugin::plugin::PluginError::Message(
                    "GPUI editor could not show its host parent",
                ))
            } else {
                ($show)(self)
            }
        }

        fn hide(&mut self) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$gui.close();
            Ok(())
        }
    };
}

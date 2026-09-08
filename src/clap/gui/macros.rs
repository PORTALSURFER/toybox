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
            let (width, height) = self.$gui.last_size().unwrap_or_else(|| {
                let (width, height) = $preferred();
                self.$gui.host_size_from_logical(width, height)
            });
            Some($crate::clack_extensions::gui::GuiSize { width, height })
        }

        fn can_resize(&mut self) -> bool {
            true
        }

        fn adjust_size(
            &mut self,
            size: $crate::clack_extensions::gui::GuiSize,
        ) -> Option<$crate::clack_extensions::gui::GuiSize> {
            let (width, height) = self.$gui.constrain_host_size(size.width, size.height);
            (width <= size.width && height <= size.height)
                .then_some($crate::clack_extensions::gui::GuiSize { width, height })
        }

        fn get_resize_hints(&mut self) -> Option<$crate::clack_extensions::gui::GuiResizeHints> {
            Some(self.$gui.resize_hints())
        }

        fn set_size(
            &mut self,
            size: $crate::clack_extensions::gui::GuiSize,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            let constrained = self.$gui.constrain_host_size(size.width, size.height);
            if constrained != (size.width, size.height) {
                return Err($crate::clack_plugin::plugin::PluginError::Message(
                    "GPUI editor rejected an unsupported host size",
                ));
            }
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

#[cfg(all(test, feature = "gpui-gui"))]
mod gpui_callback_tests {
    use crate::clack_extensions::gui::{AspectRatioStrategy, GuiSize, PluginGuiImpl};
    use crate::clack_plugin::plugin::PluginError;
    use crate::gpui_gui::GpuiHostedGui;

    struct DummyPlugin {
        gui: GpuiHostedGui,
    }

    impl DummyPlugin {
        fn new(fixed_aspect_ratio: bool) -> Self {
            let mut gui = GpuiHostedGui::new(
                "toybox-gpui-size-test",
                |_window, _cx| panic!("size tests never open a native editor"),
                800,
                500,
            )
            .with_size_contract((400, 250), (800, 500), (1200, 750));
            if fixed_aspect_ratio {
                gui = gui.with_fixed_aspect_ratio();
            }
            Self { gui }
        }
    }

    fn preferred_size() -> (u32, u32) {
        (800, 500)
    }

    fn show(_plugin: &mut DummyPlugin) -> Result<(), PluginError> {
        Ok(())
    }

    impl PluginGuiImpl for DummyPlugin {
        gpui_clap_gui_callbacks!(gui = gui, preferred_size = preferred_size, show = show);
    }

    #[test]
    fn fixed_ratio_clap_callbacks_constrain_sizes_and_reject_invalid_set_size() {
        let mut plugin = DummyPlugin::new(true);
        assert_eq!(
            plugin.get_resize_hints().unwrap().strategy,
            AspectRatioStrategy::Preserve {
                width: 800,
                height: 500,
            }
        );
        assert_eq!(
            plugin.adjust_size(GuiSize {
                width: 800,
                height: 700,
            }),
            Some(GuiSize {
                width: 800,
                height: 500,
            })
        );
        assert_eq!(
            plugin.adjust_size(GuiSize {
                width: 300,
                height: 200,
            }),
            None
        );
        assert_eq!(
            plugin.adjust_size(GuiSize {
                width: 2_000,
                height: 2_000,
            }),
            Some(GuiSize {
                width: 1_200,
                height: 750,
            })
        );

        let initial_size = plugin.get_size();
        assert!(
            plugin
                .set_size(GuiSize {
                    width: 801,
                    height: 500,
                })
                .is_err()
        );
        assert_eq!(plugin.get_size(), initial_size);

        assert!(
            plugin
                .set_size(GuiSize {
                    width: 960,
                    height: 600,
                })
                .is_ok()
        );
        assert_eq!(
            plugin.get_size(),
            Some(GuiSize {
                width: 960,
                height: 600,
            })
        );

        for width in (1..=1_600).step_by(37) {
            for height in (1..=1_000).step_by(29) {
                let requested = GuiSize { width, height };
                if let Some(adjusted) = plugin.adjust_size(requested) {
                    assert!(adjusted.width <= width && adjusted.height <= height);
                    assert!(plugin.set_size(adjusted).is_ok());
                    assert_eq!(plugin.get_size(), Some(adjusted));
                }
            }
        }
    }

    #[test]
    fn default_clap_callbacks_remain_free_aspect_ratio() {
        let mut plugin = DummyPlugin::new(false);
        assert_eq!(
            plugin.get_resize_hints().unwrap().strategy,
            AspectRatioStrategy::Disregard
        );
        assert_eq!(
            plugin.adjust_size(GuiSize {
                width: 800,
                height: 700,
            }),
            Some(GuiSize {
                width: 800,
                height: 700,
            })
        );
        assert!(
            plugin
                .set_size(GuiSize {
                    width: 800,
                    height: 700,
                })
                .is_ok()
        );
        assert_eq!(
            plugin.get_size(),
            Some(GuiSize {
                width: 800,
                height: 700,
            })
        );
    }
}

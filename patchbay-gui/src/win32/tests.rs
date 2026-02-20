#[cfg(test)]
mod tests {
    use super::{
        ShortcutBinding, ShortcutModifiers, VK_BACK, VK_DELETE, VK_END, VK_ESCAPE, VK_HOME,
        VK_LEFT, VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, WPARAM, client_size_changed,
        enforce_aspect_min, resolved_layout_size_for_resize_request,
        resolved_root_frame_resize_request,
        translate_virtual_key_to_input_char,
    };
    use crate::canvas::Size;

    #[test]
    fn aspect_enforces_min_dimensions() {
        let (w, h) = enforce_aspect_min(400, 300, 1.5);
        assert!(w >= 400);
        assert!(h >= 300);
        assert!((w as f32 / h as f32 - 1.5).abs() < 0.01);
    }

    #[test]
    fn aspect_noop_for_invalid_ratio() {
        assert_eq!(enforce_aspect_min(100, 80, 0.0), (100, 80));
    }

    #[test]
    fn client_size_change_detects_intermediate_non_aspect_step() {
        let current = Size {
            width: 420,
            height: 258,
        };
        let next = Size {
            width: 507,
            height: 269,
        };
        assert!(client_size_changed(current, next));
    }

    #[test]
    fn client_size_change_ignores_identical_size() {
        let current = Size {
            width: 640,
            height: 480,
        };
        assert!(!client_size_changed(current, current));
    }

    #[test]
    fn resize_request_uses_host_client_size_when_available() {
        let requested = Size {
            width: 500,
            height: 300,
        };
        let resolved = resolved_layout_size_for_resize_request(requested, Some((640, 400)), None);
        assert_eq!(
            resolved,
            Size {
                width: 640,
                height: 400,
            }
        );
    }

    #[test]
    fn resize_request_uses_configured_aspect_ratio_when_set() {
        let requested = Size {
            width: 500,
            height: 300,
        };
        let resolved = resolved_layout_size_for_resize_request(requested, None, Some(16.0 / 9.0));
        assert_eq!(
            resolved,
            Size {
                width: 533,
                height: 300,
            }
        );
    }

    #[test]
    fn resize_request_uses_requested_size_when_host_client_unknown() {
        let requested = Size {
            width: 777,
            height: 333,
        };
        let resolved = resolved_layout_size_for_resize_request(requested, None, None);
        assert_eq!(resolved, requested);
    }

    #[test]
    fn resize_request_prefers_host_client_with_aspect_ratio() {
        let requested = Size {
            width: 777,
            height: 333,
        };
        let resolved =
            resolved_layout_size_for_resize_request(requested, Some((640, 500)), Some(4.0 / 3.0));
        assert_eq!(
            resolved,
            Size {
                width: 640,
                height: 480,
            }
        );
    }

    #[test]
    fn root_frame_resize_request_uses_measured_size_for_auto_layout_roots() {
        let current = Size {
            width: 420,
            height: 258,
        };
        let measured = Some(Size {
            width: 512,
            height: 320,
        });
        let request = resolved_root_frame_resize_request(current, measured, None);
        assert_eq!(
            request,
            Some(Size {
                width: 512,
                height: 320,
            })
        );
    }

    #[test]
    fn root_frame_resize_request_skips_fixed_design_roots() {
        let request = resolved_root_frame_resize_request(
            Size {
                width: 420,
                height: 258,
            },
            Some(Size {
                width: 512,
                height: 320,
            }),
            Some(Size {
                width: 420,
                height: 258,
            }),
        );
        assert!(request.is_none());
    }

    #[test]
    fn translate_virtual_key_maps_text_edit_controls() {
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_BACK.0 as usize)),
            Some('\u{8}')
        );
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_DELETE.0 as usize)),
            Some('\u{7f}')
        );
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_RETURN.0 as usize)),
            Some('\r')
        );
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_ESCAPE.0 as usize)),
            Some('\u{1b}')
        );
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_TAB.0 as usize)),
            Some('\t')
        );
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_SPACE.0 as usize)),
            Some(' ')
        );
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_LEFT.0 as usize)),
            Some('\u{1c}')
        );
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_RIGHT.0 as usize)),
            Some('\u{1d}')
        );
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_HOME.0 as usize)),
            Some('\u{1e}')
        );
        assert_eq!(
            translate_virtual_key_to_input_char(WPARAM(VK_END.0 as usize)),
            Some('\u{1f}')
        );
        assert_eq!(translate_virtual_key_to_input_char(WPARAM(0x41)), None);
    }

    #[test]
    fn shortcut_modifiers_roundtrip_bits() {
        let modifiers = ShortcutModifiers::new(true, false, true);
        assert_eq!(ShortcutModifiers::from_bits(modifiers.to_bits()), modifiers);
    }

    #[test]
    fn shortcut_binding_matches_case_insensitive_key() {
        let binding = ShortcutBinding::new("save", 'S', ShortcutModifiers::default());
        assert!(binding.matches('s', ShortcutModifiers::default()));
        assert!(binding.matches('S', ShortcutModifiers::default()));
        assert!(!binding.matches('s', ShortcutModifiers::new(true, false, false)));
    }
}

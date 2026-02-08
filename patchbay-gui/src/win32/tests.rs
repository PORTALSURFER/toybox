#[cfg(test)]
mod tests {
    use super::{client_size_changed, enforce_aspect_min, resolved_layout_size_for_resize_request};
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
    fn resize_request_prefers_host_client_size_when_available() {
        let requested = Size {
            width: 500,
            height: 300,
        };
        let resolved = resolved_layout_size_for_resize_request(requested, Some((640, 400)));
        assert_eq!(
            resolved,
            Size {
                width: 640,
                height: 400,
            }
        );
    }

    #[test]
    fn resize_request_uses_requested_size_when_host_client_unknown() {
        let requested = Size {
            width: 777,
            height: 333,
        };
        let resolved = resolved_layout_size_for_resize_request(requested, None);
        assert_eq!(resolved, requested);
    }
}

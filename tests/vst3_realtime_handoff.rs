//! External-plugin coverage for the reusable VST3 realtime handoff API.

#![cfg(feature = "vst3")]
#![allow(clippy::missing_docs_in_private_items)]

use std::convert::Infallible;
use std::sync::atomic::{AtomicU32, Ordering};

use toybox::vst3::prelude::*;

/// Kickforge-style runtime whose replacement policy remains plugin-owned.
struct PluginRuntime {
    /// Runtime construction sample rate.
    sample_rate: u32,
    /// Marker representing an active DSP tail that redundant setup must retain.
    tail_marker: u32,
}

/// Copyable multi-field parameter snapshot consumed at audio boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Settings {
    /// Drive parameter.
    drive: u32,
    /// Output parameter.
    output: u32,
}

#[test]
fn downstream_plugin_needs_no_raw_pointer_handoff() {
    let (runtime_control, mut runtime_audio) = RuntimePublisher::new(PluginRuntime {
        sample_rate: 48_000,
        tail_marker: 7,
    });
    runtime_control
        .register()
        .expect("setup revision")
        .publish(PluginRuntime {
            sample_rate: 48_000,
            tail_marker: 0,
        });

    let redundant =
        runtime_audio.try_adopt(|current, candidate| current.sample_rate != candidate.sample_rate);
    assert!(matches!(
        redundant,
        RuntimeAdoption::Rejected {
            reason: RuntimeRejection::Redundant,
            ..
        }
    ));
    assert_eq!(runtime_audio.current().tail_marker, 7);
    assert_eq!(runtime_control.reclaim(), 1);

    runtime_control
        .register()
        .expect("setup revision")
        .publish(PluginRuntime {
            sample_rate: 96_000,
            tail_marker: 0,
        });
    assert!(matches!(
        runtime_audio
            .try_adopt(|current, candidate| { current.sample_rate != candidate.sample_rate }),
        RuntimeAdoption::Adopted { .. }
    ));
    assert_eq!(runtime_audio.current().sample_rate, 96_000);

    let drive = AtomicU32::new(1);
    let output = AtomicU32::new(2);
    let (state_control, mut state_audio) = CoherentStatePublisher::new(Settings {
        drive: 1,
        output: 2,
    });
    let generation = state_control
        .validate_and_publish(
            Settings {
                drive: 3,
                output: 4,
            },
            Ok::<_, Infallible>,
            |settings| {
                drive.store(settings.drive, Ordering::Relaxed);
                output.store(settings.output, Ordering::Relaxed);
            },
        )
        .expect("validated state");
    let observation = state_audio.observe();
    assert_eq!(
        observation.snapshot(),
        Settings {
            drive: 3,
            output: 4
        }
    );
    assert_eq!(observation.generation(), generation);
    assert!(observation.changed());
    assert_eq!(state_control.reclaim(), 1);
}

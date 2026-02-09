use super::{
    ParamBuilder, ParamEventContext, push_param_gesture_begin, push_param_gesture_end,
    push_param_mod, push_param_value,
};

use clack_plugin::events::Pckn;
use clack_plugin::events::io::EventBuffer;
use clack_plugin::events::spaces::CoreEventSpace;
use clack_plugin::utils::{ClapId, Cookie};

#[test]
fn push_param_events_writes_output_buffer() {
    let param_id = ClapId::new(5);
    let mut buffer = EventBuffer::new();
    let mut output = buffer.as_output();

    push_param_gesture_begin(&mut output, 0, param_id).unwrap();
    push_param_value(
        &mut output,
        param_id,
        0.75,
        ParamEventContext {
            time: 0,
            pckn: Pckn::match_all(),
            cookie: Cookie::empty(),
        },
    )
    .unwrap();
    push_param_mod(
        &mut output,
        param_id,
        0.25,
        ParamEventContext {
            time: 0,
            pckn: Pckn::match_all(),
            cookie: Cookie::empty(),
        },
    )
    .unwrap();
    push_param_gesture_end(&mut output, 0, param_id).unwrap();

    assert_eq!(buffer.len(), 4);
    let mut saw_value = false;
    let mut saw_mod = false;

    for index in 0..buffer.len() {
        let event = buffer.get(index as u32).unwrap();
        if let Some(core) = event.as_core_event() {
            match core {
                CoreEventSpace::ParamValue(value) => {
                    saw_value = value.param_id() == Some(param_id);
                }
                CoreEventSpace::ParamMod(mod_event) => {
                    saw_mod = mod_event.param_id() == Some(param_id);
                }
                _ => {}
            }
        }
    }

    assert!(saw_value);
    assert!(saw_mod);
}

#[test]
fn param_builder_sets_fields() {
    let spec = ParamBuilder::new(ClapId::new(2), b"Rate", b"Rate")
        .automatable()
        .stepped()
        .range(0.0, 10.0)
        .default(1.0)
        .build();

    assert_eq!(spec.id, ClapId::new(2));
    assert_eq!(spec.min_value, 0.0);
    assert_eq!(spec.max_value, 10.0);
    assert_eq!(spec.default_value, 1.0);
    assert!(
        spec.flags
            .contains(clack_extensions::params::ParamInfoFlags::IS_AUTOMATABLE)
    );
    assert!(
        spec.flags
            .contains(clack_extensions::params::ParamInfoFlags::IS_STEPPED)
    );
}

use clack_plugin::events::io::InputEvents;
use clack_plugin::events::spaces::CoreEventSpace;
use clack_plugin::prelude::UnknownEvent;
use clack_plugin::utils::ClapId;

/// Apply incoming automation events to a handler callback.
pub fn apply_param_events<F>(input: &InputEvents<'_>, mut apply: F)
where
    F: FnMut(ClapId, f64),
{
    for event in input {
        if let Some(CoreEventSpace::ParamValue(param)) = event.as_core_event()
            && let Some(param_id) = param.param_id()
        {
            apply(param_id, param.value());
        }
    }
}

/// Convenience helper to apply automation from a list of unknown events.
pub fn apply_param_events_from_unknown<F>(events: &[&UnknownEvent], mut apply: F)
where
    F: FnMut(ClapId, f64),
{
    for event in events {
        if let Some(CoreEventSpace::ParamValue(param)) = event.as_core_event()
            && let Some(param_id) = param.param_id()
        {
            apply(param_id, param.value());
        }
    }
}

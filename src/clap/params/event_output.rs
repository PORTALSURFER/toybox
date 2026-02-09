use clack_plugin::events::Pckn;
use clack_plugin::events::event_types::{
    ParamGestureBeginEvent, ParamGestureEndEvent, ParamModEvent, ParamValueEvent,
};
use clack_plugin::events::io::{OutputEvents, TryPushError};
use clack_plugin::utils::{ClapId, Cookie};

/// Shared metadata used when emitting CLAP parameter events.
#[derive(Clone, Copy)]
pub struct ParamEventContext {
    /// Event sample time within the current audio block.
    pub time: u32,
    /// Port/channel/key/note selector for this event.
    pub pckn: Pckn,
    /// Opaque host cookie forwarded unchanged.
    pub cookie: Cookie,
}

/// Push a CLAP parameter value event into an output event list.
pub fn push_param_value(
    output: &mut OutputEvents<'_>,
    param_id: ClapId,
    value: f64,
    context: ParamEventContext,
) -> Result<(), TryPushError> {
    output.try_push(ParamValueEvent::new(
        context.time,
        param_id,
        context.pckn,
        value,
        context.cookie,
    ))
}

/// Push a CLAP parameter modulation event into an output event list.
pub fn push_param_mod(
    output: &mut OutputEvents<'_>,
    param_id: ClapId,
    amount: f64,
    context: ParamEventContext,
) -> Result<(), TryPushError> {
    output.try_push(ParamModEvent::new(
        context.time,
        param_id,
        context.pckn,
        amount,
        context.cookie,
    ))
}

/// Push a CLAP parameter gesture begin event into an output event list.
pub fn push_param_gesture_begin(
    output: &mut OutputEvents<'_>,
    time: u32,
    param_id: ClapId,
) -> Result<(), TryPushError> {
    output.try_push(ParamGestureBeginEvent::new(time, param_id))
}

/// Push a CLAP parameter gesture end event into an output event list.
pub fn push_param_gesture_end(
    output: &mut OutputEvents<'_>,
    time: u32,
    param_id: ClapId,
) -> Result<(), TryPushError> {
    output.try_push(ParamGestureEndEvent::new(time, param_id))
}

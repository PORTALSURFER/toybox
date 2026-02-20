impl AutomationQueue {
    /// Drain queued automation events into an output buffer.
    ///
    /// The caller supplies a scratch buffer to avoid allocations in realtime
    /// threads. Events that fail to push are dropped and counted in the stats.
    /// If the queue is temporarily locked by another thread, `locked` is set
    /// and no events are drained.
    pub fn drain_to_output(
        &self,
        output: &mut OutputEvents<'_>,
        scratch: &mut Vec<AutomationEvent>,
    ) -> AutomationDrainStats {
        let Ok(mut events) = self.events.try_lock() else {
            return AutomationDrainStats {
                locked: true,
                ..AutomationDrainStats::default()
            };
        };
        if events.is_empty() {
            return AutomationDrainStats::default();
        }

        scratch.clear();
        scratch.extend(events.drain(..));
        drop(events);

        drain_scratch_to_output(scratch, output)
    }
}

/// Drain queued scratch events into host output while collecting drain stats.
fn drain_scratch_to_output(
    scratch: &mut Vec<AutomationEvent>,
    output: &mut OutputEvents<'_>,
) -> AutomationDrainStats {
    let mut stats = AutomationDrainStats {
        attempted: scratch.len(),
        ..AutomationDrainStats::default()
    };

    for event in scratch.drain(..) {
        if push_automation_event(output, event) {
            stats.pushed += 1;
        } else {
            stats.failed += 1;
        }
    }

    stats
}

/// Push one automation event to host output and return whether it succeeded.
fn push_automation_event(output: &mut OutputEvents<'_>, event: AutomationEvent) -> bool {
    match event {
        AutomationEvent::GestureBegin(param_id) => {
            push_param_gesture_begin(output, 0, param_id).is_ok()
        }
        AutomationEvent::GestureEnd(param_id) => {
            push_param_gesture_end(output, 0, param_id).is_ok()
        }
        AutomationEvent::Value(param_id, value) => push_param_value(
            output,
            param_id,
            value,
            ParamEventContext {
                time: 0,
                pckn: Pckn::match_all(),
                cookie: Cookie::empty(),
            },
        )
        .is_ok(),
    }
}

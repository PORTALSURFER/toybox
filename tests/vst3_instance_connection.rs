//! External-crate coverage for the reusable VST3 instance connection API.

#![cfg(feature = "vst3")]
#![allow(clippy::missing_docs_in_private_items)]

use std::sync::Arc;

use toybox::vst3::prelude::Steinberg::{kResultOk, tresult};
use toybox::vst3::prelude::*;

struct Shared(u32);

struct Endpoint {
    connection: InstanceConnection<Shared>,
}

impl Endpoint {
    fn new(role: InstanceConnectionRole, value: u32) -> Self {
        Self {
            connection: InstanceConnection::new(role, Arc::new(Shared(value))),
        }
    }
}

impl Class for Endpoint {
    type Interfaces = (IConnectionPoint, IToyboxSharedState);
}

impl_vst3_instance_connection!(Endpoint, connection);

#[test]
fn exported_macro_connects_external_plugin_classes() {
    let processor = ComWrapper::new(Endpoint::new(InstanceConnectionRole::Processor, 42));
    let controller = ComWrapper::new(Endpoint::new(InstanceConnectionRole::Controller, 0));
    let processor_point = processor
        .as_com_ref::<IConnectionPoint>()
        .expect("processor connection point");
    let controller_point = controller
        .as_com_ref::<IConnectionPoint>()
        .expect("controller connection point");

    let result: tresult = unsafe { controller_point.connect(processor_point.as_ptr()) };
    assert_eq!(result, kResultOk);
    assert_eq!(controller.connection.shared().0, 42);
}

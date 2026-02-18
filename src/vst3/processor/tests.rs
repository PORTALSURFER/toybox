//! Tests for stereo process buffer extraction.

use std::mem;
use std::ptr;
use toybox_vst3_ffi::Steinberg::Vst::ProcessData;
use toybox_vst3_ffi::Steinberg::Vst::{AudioBusBuffers, AudioBusBuffers__type0};

struct StereoProcessFixture {
    process_data: ProcessData,
    _input_left: Vec<f32>,
    _input_right: Vec<f32>,
    _output_left: Vec<f32>,
    _output_right: Vec<f32>,
    _input_channel_buffers: Vec<*mut f32>,
    _output_channel_buffers: Vec<*mut f32>,
    _input_buses: Vec<AudioBusBuffers>,
    _output_buses: Vec<AudioBusBuffers>,
}

fn zeroed_process_data() -> ProcessData {
    // SAFETY: This produces a valid zeroed baseline for `ProcessData` test
    // scaffolding; individual fields are overwritten for each scenario.
    unsafe { mem::zeroed() }
}

fn stereo_process_data_with_samples(samples: usize) -> StereoProcessFixture {
    let mut input_left = vec![1.0; samples];
    let mut input_right = vec![0.5; samples];
    let mut output_left = vec![0.0; samples];
    let mut output_right = vec![0.0; samples];

    let mut input_channel_buffers = vec![input_left.as_mut_ptr(), input_right.as_mut_ptr()];
    let mut output_channel_buffers = vec![output_left.as_mut_ptr(), output_right.as_mut_ptr()];

    let input_bus = AudioBusBuffers {
        numChannels: 2,
        silenceFlags: 0,
        __field0: AudioBusBuffers__type0 {
            channelBuffers32: input_channel_buffers.as_mut_ptr(),
        },
    };
    let output_bus = AudioBusBuffers {
        numChannels: 2,
        silenceFlags: 0,
        __field0: AudioBusBuffers__type0 {
            channelBuffers32: output_channel_buffers.as_mut_ptr(),
        },
    };

    let mut input_buses = vec![input_bus];
    let mut output_buses = vec![output_bus];

    let mut data = zeroed_process_data();
    data.numInputs = 1;
    data.numOutputs = 1;
    data.numSamples = i32::try_from(samples).expect("sample count must fit i32");
    data.inputs = input_buses.as_mut_ptr();
    data.outputs = output_buses.as_mut_ptr();

    StereoProcessFixture {
        process_data: data,
        _input_left: input_left,
        _input_right: input_right,
        _output_left: output_left,
        _output_right: output_right,
        _input_channel_buffers: input_channel_buffers,
        _output_channel_buffers: output_channel_buffers,
        _input_buses: input_buses,
        _output_buses: output_buses,
    }
}

#[test]
fn stereo_f32_buffers_extracts_stereo_buffers() {
    let fixture = stereo_process_data_with_samples(64);

    let buffers = unsafe { crate::vst3::processor::stereo_f32_buffers(&fixture.process_data) }
        .expect("stereo process data should produce buffers");

    assert_eq!(buffers.num_samples, 64);
    assert_eq!(buffers.input_left.len(), 64);
    assert_eq!(buffers.input_right.len(), 64);
    assert_eq!(buffers.output_left.len(), 64);
    assert_eq!(buffers.output_right.len(), 64);
}

#[test]
fn stereo_f32_buffers_rejects_non_stereo_bus_layout() {
    let mut fixture = stereo_process_data_with_samples(0);
    fixture.process_data.numInputs = 2;
    assert!(unsafe { crate::vst3::processor::stereo_f32_buffers(&fixture.process_data) }.is_none());
}

#[test]
fn stereo_f32_buffers_rejects_non_stereo_input_channels() {
    let mut fixture = stereo_process_data_with_samples(64);
    fixture._input_buses[0].numChannels = 1;
    assert!(unsafe { crate::vst3::processor::stereo_f32_buffers(&fixture.process_data) }.is_none());
}

#[test]
fn stereo_f32_buffers_rejects_non_stereo_output_channels() {
    let mut fixture = stereo_process_data_with_samples(64);
    fixture._output_buses[0].numChannels = 1;
    assert!(unsafe { crate::vst3::processor::stereo_f32_buffers(&fixture.process_data) }.is_none());
}

#[test]
fn stereo_f32_buffers_rejects_null_input_channel_buffer() {
    let mut fixture = stereo_process_data_with_samples(64);
    fixture._input_channel_buffers[0] = ptr::null_mut();
    assert!(unsafe { crate::vst3::processor::stereo_f32_buffers(&fixture.process_data) }.is_none());
}

#[test]
fn stereo_f32_buffers_rejects_null_output_channel_buffer() {
    let mut fixture = stereo_process_data_with_samples(64);
    fixture._output_channel_buffers[0] = ptr::null_mut();
    assert!(unsafe { crate::vst3::processor::stereo_f32_buffers(&fixture.process_data) }.is_none());
}

#[test]
fn stereo_f32_buffers_rejects_missing_bus_pointers() {
    let mut fixture = stereo_process_data_with_samples(64);
    fixture.process_data.inputs = ptr::null_mut();
    fixture.process_data.outputs = ptr::null_mut();
    assert!(unsafe { crate::vst3::processor::stereo_f32_buffers(&fixture.process_data) }.is_none());
}

#[test]
fn stereo_f32_buffers_rejects_negative_sample_count() {
    let mut fixture = stereo_process_data_with_samples(64);
    fixture.process_data.numSamples = -1;
    assert!(unsafe { crate::vst3::processor::stereo_f32_buffers(&fixture.process_data) }.is_none());
}

#[test]
fn stereo_f32_buffers_rejects_missing_input_bus_pointer() {
    let mut data = stereo_process_data_with_samples(64);
    data.process_data.inputs = ptr::null_mut();
    assert!(unsafe { crate::vst3::processor::stereo_f32_buffers(&data.process_data) }.is_none());
}

#[test]
fn stereo_f32_buffers_rejects_missing_output_bus_pointer() {
    let mut data = stereo_process_data_with_samples(64);
    data.process_data.outputs = ptr::null_mut();
    assert!(unsafe { crate::vst3::processor::stereo_f32_buffers(&data.process_data) }.is_none());
}

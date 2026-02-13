//! Audio processing helpers for VST3 `ProcessData` blocks.

use std::slice;

use toybox_vst3_ffi::Steinberg::Vst::{AudioBusBuffers, ProcessData};
use toybox_vst3_ffi::Steinberg::{kResultOk, tresult};

/// Borrowed stereo f32 buffers extracted from VST3 process data.
pub struct StereoAudioBuffers<'a> {
    /// Number of samples in the current process block.
    pub num_samples: usize,
    /// Left input channel data.
    pub input_left: &'a [f32],
    /// Right input channel data.
    pub input_right: &'a [f32],
    /// Left output channel data.
    pub output_left: &'a mut [f32],
    /// Right output channel data.
    pub output_right: &'a mut [f32],
}

/// Expected number of channels for stereo audio processing.
const STEREO_CHANNEL_COUNT: usize = 2;

/// Build stereo f32 process slices from a VST3 process block.
///
/// Returns `None` when the block is not exactly one stereo input bus and one
/// stereo output bus with f32 sample buffers.
///
/// # Safety
///
/// `data` must be a valid process block for the duration of the returned
/// borrow.
pub unsafe fn stereo_f32_buffers<'a>(data: &'a ProcessData) -> Option<StereoAudioBuffers<'a>> {
    if !has_stereo_bus_layout(data) {
        return None;
    }

    let num_samples = usize::try_from(data.numSamples).ok()?;
    let (input_buses, output_buses) = unsafe { read_bus_lists(data)? };

    let (input_left_ptr, input_right_ptr) = unsafe { stereo_channel_ptrs(&input_buses[0])? };
    let (output_left_ptr, output_right_ptr) = unsafe { stereo_channel_ptrs(&output_buses[0])? };

    let input_left = unsafe { slice::from_raw_parts(input_left_ptr, num_samples) };
    let input_right = unsafe { slice::from_raw_parts(input_right_ptr, num_samples) };
    let output_left = unsafe { slice::from_raw_parts_mut(output_left_ptr, num_samples) };
    let output_right = unsafe { slice::from_raw_parts_mut(output_right_ptr, num_samples) };

    Some(StereoAudioBuffers {
        num_samples,
        input_left,
        input_right,
        output_left,
        output_right,
    })
}

/// Validate the bus layout for stereo processing.
///
/// The helper requires one input bus and one output bus, and both bus pointers
/// must be non-null.
#[inline]
fn has_stereo_bus_layout(data: &ProcessData) -> bool {
    data.numInputs == 1 && data.numOutputs == 1 && !data.inputs.is_null() && !data.outputs.is_null()
}

/// Read immutable bus slices from the VST3 process block.
///
/// Returns `None` when either bus count is negative or otherwise invalid for
/// safe slicing.
#[inline]
unsafe fn read_bus_lists(data: &ProcessData) -> Option<(&[AudioBusBuffers], &[AudioBusBuffers])> {
    let input_bus_count = usize::try_from(data.numInputs).ok()?;
    let output_bus_count = usize::try_from(data.numOutputs).ok()?;
    Some((
        unsafe { slice::from_raw_parts(data.inputs, input_bus_count) },
        unsafe { slice::from_raw_parts(data.outputs, output_bus_count) },
    ))
}

/// Extract mutable channel pointers for stereo data layout.
///
/// Returns `None` if the bus is not stereo or either channel pointer is null.
#[inline]
unsafe fn stereo_channel_ptrs(bus: &AudioBusBuffers) -> Option<(*mut f32, *mut f32)> {
    if bus.numChannels != STEREO_CHANNEL_COUNT as i32 {
        return None;
    }

    let channels =
        unsafe { slice::from_raw_parts(bus.__field0.channelBuffers32, STEREO_CHANNEL_COUNT) };
    if channels.iter().any(|channel| channel.is_null()) {
        return None;
    }

    Some((channels[0], channels[1]))
}

/// Return the VST3 success status used for a normal process block completion.
pub const fn process_ok() -> tresult {
    kResultOk
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    use std::ptr;
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

        let buffers = unsafe { super::stereo_f32_buffers(&fixture.process_data) }
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
        assert!(unsafe { super::stereo_f32_buffers(&fixture.process_data) }.is_none());
    }

    #[test]
    fn stereo_f32_buffers_rejects_non_stereo_input_channels() {
        let mut fixture = stereo_process_data_with_samples(64);
        fixture._input_buses[0].numChannels = 1;
        assert!(unsafe { super::stereo_f32_buffers(&fixture.process_data) }.is_none());
    }

    #[test]
    fn stereo_f32_buffers_rejects_non_stereo_output_channels() {
        let mut fixture = stereo_process_data_with_samples(64);
        fixture._output_buses[0].numChannels = 1;
        assert!(unsafe { super::stereo_f32_buffers(&fixture.process_data) }.is_none());
    }

    #[test]
    fn stereo_f32_buffers_rejects_null_input_channel_buffer() {
        let mut fixture = stereo_process_data_with_samples(64);
        fixture._input_channel_buffers[0] = ptr::null_mut();
        assert!(unsafe { super::stereo_f32_buffers(&fixture.process_data) }.is_none());
    }

    #[test]
    fn stereo_f32_buffers_rejects_null_output_channel_buffer() {
        let mut fixture = stereo_process_data_with_samples(64);
        fixture._output_channel_buffers[0] = ptr::null_mut();
        assert!(unsafe { super::stereo_f32_buffers(&fixture.process_data) }.is_none());
    }

    #[test]
    fn stereo_f32_buffers_rejects_missing_bus_pointers() {
        let mut fixture = stereo_process_data_with_samples(64);
        fixture.process_data.inputs = ptr::null_mut();
        fixture.process_data.outputs = ptr::null_mut();
        assert!(unsafe { super::stereo_f32_buffers(&fixture.process_data) }.is_none());
    }

    #[test]
    fn stereo_f32_buffers_rejects_negative_sample_count() {
        let mut fixture = stereo_process_data_with_samples(64);
        fixture.process_data.numSamples = -1;
        assert!(unsafe { super::stereo_f32_buffers(&fixture.process_data) }.is_none());
    }

    #[test]
    fn stereo_f32_buffers_rejects_missing_input_bus_pointer() {
        let mut data = stereo_process_data_with_samples(64);
        data.process_data.inputs = ptr::null_mut();
        assert!(unsafe { super::stereo_f32_buffers(&data.process_data) }.is_none());
    }

    #[test]
    fn stereo_f32_buffers_rejects_missing_output_bus_pointer() {
        let mut data = stereo_process_data_with_samples(64);
        data.process_data.outputs = ptr::null_mut();
        assert!(unsafe { super::stereo_f32_buffers(&data.process_data) }.is_none());
    }
}

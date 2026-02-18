//! Core helper types and functions for extracting stereo bus buffers.

use std::slice;

use toybox_vst3_ffi::Steinberg::Vst::{AudioBusBuffers, ProcessData};
use toybox_vst3_ffi::Steinberg::{kResultOk, tresult};

/// Borrowed stereo f32 buffers extracted from a VST3 process block.
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

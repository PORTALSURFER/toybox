//! Audio processing helpers for VST3 `ProcessData` blocks.

use std::slice;

use toybox_vst3_ffi::Steinberg::Vst::ProcessData;
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
    if data.numInputs != 1 || data.numOutputs != 1 {
        return None;
    }

    if data.inputs.is_null() || data.outputs.is_null() {
        return None;
    }

    let num_samples = usize::try_from(data.numSamples).ok()?;

    let input_bus_count = usize::try_from(data.numInputs).ok()?;
    let output_bus_count = usize::try_from(data.numOutputs).ok()?;

    let input_buses = unsafe { slice::from_raw_parts(data.inputs, input_bus_count) };
    let output_buses = unsafe { slice::from_raw_parts(data.outputs, output_bus_count) };

    if input_buses[0].numChannels != 2 || output_buses[0].numChannels != 2 {
        return None;
    }

    let input_channels = unsafe {
        slice::from_raw_parts(
            input_buses[0].__field0.channelBuffers32,
            input_buses[0].numChannels as usize,
        )
    };
    let output_channels = unsafe {
        slice::from_raw_parts(
            output_buses[0].__field0.channelBuffers32,
            output_buses[0].numChannels as usize,
        )
    };

    if input_channels.iter().any(|channel| channel.is_null())
        || output_channels.iter().any(|channel| channel.is_null())
    {
        return None;
    }

    let input_left = unsafe { slice::from_raw_parts(input_channels[0], num_samples) };
    let input_right = unsafe { slice::from_raw_parts(input_channels[1], num_samples) };
    let output_left = unsafe { slice::from_raw_parts_mut(output_channels[0], num_samples) };
    let output_right = unsafe { slice::from_raw_parts_mut(output_channels[1], num_samples) };

    Some(StereoAudioBuffers {
        num_samples,
        input_left,
        input_right,
        output_left,
        output_right,
    })
}

/// Return the VST3 success status used for a normal process block completion.
pub const fn process_ok() -> tresult {
    kResultOk
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    fn zeroed_process_data() -> ProcessData {
        // SAFETY: This produces a valid zeroed baseline for `ProcessData` test
        // scaffolding; individual fields are overwritten for each scenario.
        unsafe { mem::zeroed() }
    }

    #[test]
    fn stereo_f32_buffers_rejects_non_stereo_bus_layout() {
        let mut data = zeroed_process_data();
        data.numInputs = 2;
        data.numOutputs = 1;
        data.numSamples = 0;
        assert!(unsafe { super::stereo_f32_buffers(&data) }.is_none());
    }

    #[test]
    fn stereo_f32_buffers_rejects_missing_bus_pointers() {
        let mut data = zeroed_process_data();
        data.numInputs = 1;
        data.numOutputs = 1;
        data.numSamples = 64;
        data.inputs = std::ptr::null();
        data.outputs = std::ptr::null();
        assert!(unsafe { super::stereo_f32_buffers(&data) }.is_none());
    }
}

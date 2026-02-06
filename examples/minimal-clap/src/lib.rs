//! Minimal CLAP gain plugin showcasing toybox helpers.

#![deny(clippy::missing_docs_in_private_items, missing_docs, warnings)]

use std::fmt::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use toybox::clack_plugin::stream::{InputStream, OutputStream};
use toybox::clap::prelude::*;

/// CLAP parameter id for the gain control.
const PARAM_GAIN_ID: ClapId = ClapId::new(0);

/// Default gain value.
const DEFAULT_GAIN: f32 = 1.0;
/// Minimum gain value.
const MIN_GAIN: f32 = 0.0;
/// Maximum gain value.
const MAX_GAIN: f32 = 2.0;
/// State payload magic (`MGST`).
const STATE_MAGIC: u32 = u32::from_le_bytes(*b"MGST");
/// State payload version.
const STATE_VERSION: u32 = 1;

/// Minimal CLAP plugin type.
pub struct MinimalGainPlugin;

impl Plugin for MinimalGainPlugin {
    type AudioProcessor<'a> = MinimalGainAudioProcessor<'a>;
    type Shared<'a> = MinimalGainShared;
    type MainThread<'a> = MinimalGainMainThread<'a>;

    fn declare_extensions(
        builder: &mut PluginExtensions<Self>,
        _shared: Option<&Self::Shared<'_>>,
    ) {
        register_default_extensions(builder);
    }
}

impl DefaultPluginFactory for MinimalGainPlugin {
    fn get_descriptor() -> PluginDescriptor {
        PluginDescriptor::new("com.toybox.minimal-gain", "Toybox Minimal Gain")
    }

    fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
        Ok(MinimalGainShared {
            params: Arc::new(GainParams::new()),
        })
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(MinimalGainMainThread { shared })
    }
}

/// Shared state between threads.
pub struct MinimalGainShared {
    /// Gain parameter state.
    pub params: Arc<GainParams>,
}

impl PluginShared<'_> for MinimalGainShared {}

/// Main-thread state for the minimal gain plugin.
pub struct MinimalGainMainThread<'a> {
    /// Shared parameter storage.
    shared: &'a MinimalGainShared,
}

impl<'a> PluginMainThread<'a, MinimalGainShared> for MinimalGainMainThread<'a> {}

impl PluginAudioPortsImpl for MinimalGainMainThread<'_> {
    fn count(&mut self, _is_input: bool) -> u32 {
        1
    }

    fn get(&mut self, index: u32, _is_input: bool, writer: &mut AudioPortInfoWriter) {
        if index != 0 {
            return;
        }
        writer.set(&AudioPortInfo {
            id: ClapId::new(0),
            name: b"main",
            channel_count: 2,
            flags: AudioPortFlags::IS_MAIN,
            port_type: Some(AudioPortType::STEREO),
            in_place_pair: None,
        })
    }
}

impl PluginMainThreadParams for MinimalGainMainThread<'_> {
    fn count(&mut self) -> u32 {
        1
    }

    fn get_info(&mut self, param_index: u32, info: &mut ParamInfoWriter) {
        if param_index != 0 {
            return;
        }

        let spec = ParamBuilder::new(PARAM_GAIN_ID, b"Gain", b"Gain")
            .automatable()
            .range(MIN_GAIN as f64, MAX_GAIN as f64)
            .default(DEFAULT_GAIN as f64)
            .build();
        spec.write(info);
    }

    fn get_value(&mut self, param_id: ClapId) -> Option<f64> {
        if param_id == PARAM_GAIN_ID {
            Some(self.shared.params.gain() as f64)
        } else {
            None
        }
    }

    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result {
        if param_id == PARAM_GAIN_ID {
            write!(writer, "{value:.2}x")
        } else {
            Err(std::fmt::Error)
        }
    }

    fn text_to_value(&mut self, param_id: ClapId, text: &std::ffi::CStr) -> Option<f64> {
        if param_id != PARAM_GAIN_ID {
            return None;
        }
        let raw = text.to_str().ok()?.trim().trim_end_matches('x');
        let value: f64 = raw.parse().ok()?;
        Some(value.clamp(MIN_GAIN as f64, MAX_GAIN as f64))
    }

    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut OutputEvents,
    ) {
        apply_param_events(input_parameter_changes, |param_id, value| {
            if param_id == PARAM_GAIN_ID {
                self.shared.params.set_gain(value as f32);
            }
        });
    }
}

impl PluginStateImpl for MinimalGainMainThread<'_> {
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError> {
        let payload = self.shared.params.gain().to_le_bytes();
        write_versioned_payload(output, STATE_MAGIC, STATE_VERSION, &payload)?;
        Ok(())
    }

    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError> {
        let state = read_versioned_payload(input, STATE_MAGIC, &[STATE_VERSION])?;
        if state.payload.len() != 4 {
            return Err(PluginError::Message("Invalid minimal-gain state payload"));
        }
        let value = f32::from_le_bytes([
            state.payload[0],
            state.payload[1],
            state.payload[2],
            state.payload[3],
        ]);
        if !value.is_finite() {
            return Err(PluginError::Message("Invalid minimal-gain state payload"));
        }
        self.shared.params.set_gain(value.clamp(MIN_GAIN, MAX_GAIN));
        Ok(())
    }
}

/// Audio processor for the minimal gain plugin.
pub struct MinimalGainAudioProcessor<'a> {
    /// Shared parameter state.
    shared: &'a MinimalGainShared,
}

impl<'a> PluginAudioProcessor<'a, MinimalGainShared, MinimalGainMainThread<'a>>
    for MinimalGainAudioProcessor<'a>
{
    fn activate(
        _host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut MinimalGainMainThread<'a>,
        shared: &'a MinimalGainShared,
        _audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self { shared })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        apply_param_events(events.input, |param_id, value| {
            if param_id == PARAM_GAIN_ID {
                self.shared.params.set_gain(value as f32);
            }
        });

        let gain = self.shared.params.gain();
        for mut port_pair in &mut audio {
            let Some(mut channels) = port_pair.channels()?.into_f32() else {
                continue;
            };
            for channel in channels.iter_mut() {
                match channel {
                    ChannelPair::InputOnly(_) => {}
                    ChannelPair::OutputOnly(buf) => buf.fill(0.0),
                    ChannelPair::InputOutput(input, output) => {
                        for (input, output) in input.iter().zip(output.iter_mut()) {
                            *output = input * gain;
                        }
                    }
                    ChannelPair::InPlace(buf) => {
                        for sample in buf.iter_mut() {
                            *sample *= gain;
                        }
                    }
                }
            }
        }

        Ok(ProcessStatus::Continue)
    }
}

impl PluginAudioProcessorParams for MinimalGainAudioProcessor<'_> {
    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut OutputEvents,
    ) {
        apply_param_events(input_parameter_changes, |param_id, value| {
            if param_id == PARAM_GAIN_ID {
                self.shared.params.set_gain(value as f32);
            }
        });
    }
}

/// Parameter storage for the minimal gain plugin.
pub struct GainParams {
    /// Gain value stored atomically.
    gain: AtomicF32,
}

impl GainParams {
    /// Create a new parameter set with defaults applied.
    pub fn new() -> Self {
        Self {
            gain: AtomicF32::new(DEFAULT_GAIN),
        }
    }

    /// Read the current gain value.
    pub fn gain(&self) -> f32 {
        self.gain.load(Ordering::Relaxed)
    }

    /// Update the gain value.
    pub fn set_gain(&self, value: f32) {
        self.gain.store(value, Ordering::Relaxed);
    }
}

/// An atomic `f32` backed by an `AtomicU32`.
struct AtomicF32 {
    /// Packed float bits stored atomically.
    value: AtomicU32,
}

impl AtomicF32 {
    /// Create a new atomic float.
    fn new(value: f32) -> Self {
        Self {
            value: AtomicU32::new(u32::from_ne_bytes(value.to_ne_bytes())),
        }
    }

    /// Store a new float value.
    fn store(&self, value: f32, ordering: Ordering) {
        self.value
            .store(u32::from_ne_bytes(value.to_ne_bytes()), ordering);
    }

    /// Load the current float value.
    fn load(&self, ordering: Ordering) -> f32 {
        f32::from_ne_bytes(self.value.load(ordering).to_ne_bytes())
    }
}

toybox::clap_plugin_entry!(MinimalGainPlugin);

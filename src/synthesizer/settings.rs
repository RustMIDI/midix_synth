use crate::prelude::*;

/// Specifies a set of parameters for synthesis.
#[derive(Copy, Clone)]
pub struct SynthesizerSettings {
    /// The sample rate for synthesis.
    pub sample_rate: i32,
    /// The block size for rendering waveform.
    pub block_size: usize,
    /// The number of maximum polyphony.
    pub maximum_polyphony: usize,
    /// The value indicating whether reverb and chorus are enabled.
    pub enable_reverb_and_chorus: bool,
}

impl Default for SynthesizerSettings {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            block_size: 64,
            maximum_polyphony: 64,
            enable_reverb_and_chorus: true,
        }
    }
}

impl SynthesizerSettings {
    /// Initializes a new instance of synthesizer settings.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate for synthesis.
    pub fn new(sample_rate: i32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }

    pub(crate) fn validate(&self) -> Result<(), SynthesizerError> {
        SynthesizerSettings::check_sample_rate(self.sample_rate)?;
        SynthesizerSettings::check_block_size(self.block_size)?;
        SynthesizerSettings::check_maximum_polyphony(self.maximum_polyphony)?;

        Ok(())
    }

    fn check_sample_rate(value: i32) -> Result<(), SynthesizerError> {
        if !(16_000..=192_000).contains(&value) {
            return Err(SynthesizerError::SampleRateOutOfRange(value));
        }

        Ok(())
    }

    fn check_block_size(value: usize) -> Result<(), SynthesizerError> {
        if !(8..=1024).contains(&value) {
            return Err(SynthesizerError::BlockSizeOutOfRange(value));
        }

        Ok(())
    }

    fn check_maximum_polyphony(value: usize) -> Result<(), SynthesizerError> {
        if !(8..=256).contains(&value) {
            return Err(SynthesizerError::MaximumPolyphonyOutOfRange(value));
        }

        Ok(())
    }
}

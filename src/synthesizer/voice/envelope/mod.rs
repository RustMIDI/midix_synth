mod modulation;
pub(super) use modulation::*;

mod volume;
pub(super) use volume::*;

mod common;
//use common::*;

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum EnvelopeStage {
    Delay,
    Attack,
    Hold,
    Decay,
    Release { time: f64, level: f32 },
}

impl EnvelopeStage {
    pub fn release(processed_sample_count: usize, sample_rate: i32, level: f32) -> Self {
        Self::Release {
            time: processed_sample_count as f64 / sample_rate as f64,
            level,
        }
    }
    // pub fn next(self) -> EnvelopeStage {
    //     use EnvelopeStage::*;
    //     match self {
    //         Delay => Attack,
    //         Attack => Hold,
    //         Hold => Decay,
    //         Decay => Release,
    //         Release => Release,
    //     }
    // }
    // /// will update the value and return Some if the value should proceed
    // pub fn update_value(&self, value: &mut f32, attack_slope: f32, current_time: f32, attack_start_time) -> Option<f32> {
    //     match self {
    //         EnvelopeStage::Delay => {
    //             *value = 0.;
    //             Some(0.)
    //         }
    //         EnvelopeStage::Attack => {
    //             *value = (self.attack_slope * (current_time - self.attack_start_time)) as f32;
    //             Some(*value)
    //         }
    //         EnvelopeStage::Hold => {
    //             *value = 1.;
    //             Some(*value)
    //         }
    //         EnvelopeStage::Decay => {
    //             *value = ((self.decay_slope * (self.decay_end_time - current_time)) as f32)
    //                 .max(self.sustain_level);
    //             (*value > utils::NON_AUDIBLE).then_some(*value)
    //         }
    //         EnvelopeStage::Release => {
    //             *value = ((self.release_level as f64
    //                 * self.release_slope
    //                 * (self.release_end_time - current_time)) as f32)
    //                 .max(0.);

    //             (*value > utils::NON_AUDIBLE).then_some(*value)
    //         }
    //     }
    // }
}

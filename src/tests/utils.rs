#![allow(dead_code)]

use std::{fs, io::Cursor, sync::Arc};

use midix::prelude::ChannelVoiceMessage;

/// Configuration for synthesizer comparison tests
#[derive(Debug, Clone)]
pub struct ComparisonConfig {
    /// Sample rate for both synthesizers
    pub sample_rate: i32,
    /// Number of frames per render call
    pub frames_per_render: usize,
    /// Tolerance for floating point comparison
    pub epsilon: f32,
    /// Whether to print detailed output
    pub verbose: bool,
    /// Maximum number of differences to report
    pub max_differences_to_report: usize,
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            frames_per_render: 512,
            epsilon: 1e-6,
            verbose: false,
            max_differences_to_report: 10,
        }
    }
}

/// Result of comparing two waveforms
#[derive(Debug)]
pub struct ComparisonResult {
    pub total_samples: usize,
    pub max_difference: f32,
    pub differences: Vec<SampleDifference>,
    pub passed: bool,
}

#[derive(Debug, Clone)]
pub struct SampleDifference {
    pub sample_index: usize,
    pub midix_value: f32,
    pub rusty_value: f32,
    pub difference: f32,
}

/// Test harness for comparing midix and RustySynth
pub struct SynthesizerComparison {
    pub midix_synth: crate::prelude::Synthesizer,
    pub rusty_synth: rustysynth::Synthesizer,
    pub config: ComparisonConfig,
    pub mleft: Vec<f32>,
    pub mright: Vec<f32>,
    pub rleft: Vec<f32>,
    pub rright: Vec<f32>,
}

impl SynthesizerComparison {
    /// Create a new comparison harness with the given soundfont and configuration
    pub fn new(
        soundfont_path: &str,
        config: ComparisonConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = fs::read(soundfont_path)?;

        let midix_soundfont = crate::prelude::SoundFont::new(&mut Cursor::new(bytes.clone()))?;
        let rs_soundfont = rustysynth::SoundFont::new(&mut Cursor::new(bytes.clone()))?;

        let midix_synth = crate::prelude::Synthesizer::new(
            Arc::new(midix_soundfont),
            &crate::prelude::SynthesizerSettings::new(config.sample_rate),
        )?;

        let rusty_synth = rustysynth::Synthesizer::new(
            &Arc::new(rs_soundfont),
            &rustysynth::SynthesizerSettings::new(config.sample_rate),
        )?;

        let buffer_size = config.frames_per_render;

        Ok(Self {
            midix_synth,
            rusty_synth,
            config,
            mleft: vec![0.0; buffer_size],
            mright: vec![0.0; buffer_size],
            rleft: vec![0.0; buffer_size],
            rright: vec![0.0; buffer_size],
        })
    }

    /// Process MIDI message for both synthesizers
    #[allow(dead_code)]
    pub fn process_midi_message(&mut self, message: ChannelVoiceMessage) {
        self.midix_synth.process_midi_message(message);
        let data1 = message.data_1_byte() as i32;
        let data2 = message.data_2_byte().unwrap_or(0) as i32;
        let channel = (message.status() & 0b0000_1111) as i32;
        let command = (message.status() & 0b1111_0000) as i32;
        self.rusty_synth
            .process_midi_message(channel, command, data1, data2);
    }

    // /// Set pitch bend for both synthesizers
    // pub fn pitch_bend(&mut self, channel: u8, value: u16) {
    //     let lsb = (value & 0x7F) as u8;
    //     let msb = ((value >> 7) & 0x7F) as u8;
    //     self.midix_synth
    //         .process_midi_message(0xE0 | channel, lsb, msb);
    //     self.rusty_synth
    //         .process_midi_message(channel as i32, 0xE0, lsb as i32, msb as i32);
    // }

    /// Reset both synthesizers
    pub fn reset(&mut self) {
        self.midix_synth.reset();
        self.rusty_synth.reset();
    }

    /// Render and compare one frame
    pub fn render_and_compare(&mut self) -> ComparisonResult {
        // Render both synthesizers
        self.midix_synth.render(&mut self.mleft, &mut self.mright);
        self.rusty_synth.render(&mut self.rleft, &mut self.rright);

        // Compare outputs
        self.compare_buffers(&self.mleft, &self.rleft, "left")
    }

    /// Render and compare multiple frames
    pub fn render_and_compare_frames(&mut self, num_frames: usize) -> ComparisonResult {
        let mut all_differences = Vec::new();
        let mut max_difference = 0.0f32;
        let mut total_samples = 0;

        for frame_idx in 0..num_frames {
            // Render both synthesizers
            self.midix_synth.render(&mut self.mleft, &mut self.mright);
            self.rusty_synth.render(&mut self.rleft, &mut self.rright);

            // Compare left channel
            for (i, (m, r)) in self.mleft.iter().zip(self.rleft.iter()).enumerate() {
                let diff = (m - r).abs();
                if diff > self.config.epsilon {
                    all_differences.push(SampleDifference {
                        sample_index: frame_idx * self.config.frames_per_render + i,
                        midix_value: *m,
                        rusty_value: *r,
                        difference: diff,
                    });
                }
                max_difference = max_difference.max(diff);
                total_samples += 1;
            }

            // Compare right channel
            for (i, (m, r)) in self.mright.iter().zip(self.rright.iter()).enumerate() {
                let diff = (m - r).abs();
                if diff > self.config.epsilon {
                    all_differences.push(SampleDifference {
                        sample_index: frame_idx * self.config.frames_per_render
                            + i
                            + self.mleft.len(),
                        midix_value: *m,
                        rusty_value: *r,
                        difference: diff,
                    });
                }
                max_difference = max_difference.max(diff);
                total_samples += 1;
            }
        }

        let passed = all_differences.is_empty();

        if self.config.verbose {
            self.print_comparison_report(&all_differences, max_difference, total_samples);
        }

        ComparisonResult {
            total_samples,
            max_difference,
            differences: all_differences,
            passed,
        }
    }

    /// Compare two buffers
    fn compare_buffers(
        &self,
        midix: &[f32],
        rusty: &[f32],
        _channel_name: &str,
    ) -> ComparisonResult {
        let mut differences = Vec::new();
        let mut max_difference = 0.0f32;

        for (i, (m, r)) in midix.iter().zip(rusty.iter()).enumerate() {
            let diff = (m - r).abs();
            if diff > self.config.epsilon {
                differences.push(SampleDifference {
                    sample_index: i,
                    midix_value: *m,
                    rusty_value: *r,
                    difference: diff,
                });
            }
            max_difference = max_difference.max(diff);
        }

        let passed = differences.is_empty();

        ComparisonResult {
            total_samples: midix.len(),
            max_difference,
            differences,
            passed,
        }
    }

    /// Print a detailed comparison report
    fn print_comparison_report(
        &self,
        differences: &[SampleDifference],
        max_difference: f32,
        total_samples: usize,
    ) {
        println!("\n=== Comparison Report ===");
        println!("Total samples compared: {total_samples}");
        println!("Maximum difference: {max_difference:.9e}");
        println!(
            "Samples exceeding epsilon ({}): {}",
            self.config.epsilon,
            differences.len()
        );

        if !differences.is_empty() {
            println!(
                "\nFirst {} differences:",
                self.config.max_differences_to_report.min(differences.len())
            );
            for (idx, diff) in differences
                .iter()
                .take(self.config.max_differences_to_report)
                .enumerate()
            {
                println!(
                    "  [{}] Sample {}: midix={:.9}, rusty={:.9}, diff={:.9e}",
                    idx, diff.sample_index, diff.midix_value, diff.rusty_value, diff.difference
                );
            }

            // Find and show largest differences
            let mut sorted_diffs = differences.to_vec();
            sorted_diffs.sort_by(|a, b| b.difference.partial_cmp(&a.difference).unwrap());

            if sorted_diffs.len() > self.config.max_differences_to_report {
                println!(
                    "\nTop {} largest differences:",
                    self.config
                        .max_differences_to_report
                        .min(sorted_diffs.len())
                );
                for (idx, diff) in sorted_diffs
                    .iter()
                    .take(self.config.max_differences_to_report)
                    .enumerate()
                {
                    println!(
                        "  [{}] Sample {}: midix={:.9}, rusty={:.9}, diff={:.9e}",
                        idx, diff.sample_index, diff.midix_value, diff.rusty_value, diff.difference
                    );
                }
            }
        }
    }
}

pub struct TestAction {
    pub name: String,
    pub frames_before_action: usize,
    pub action: Box<dyn FnMut(&mut SynthesizerComparison)>,
    pub frames_after_action: usize,
}
impl TestAction {
    pub fn new(
        name: String,
        action: ChannelVoiceMessage,
        frames_before_action: usize,
        frames_after_action: usize,
    ) -> Self {
        Self {
            name,
            frames_before_action,
            action: Box::new(move |synth| synth.process_midi_message(action)),
            frames_after_action,
        }
    }
    /// Run the test scenario and return the result
    pub fn run(&mut self, synth: &mut SynthesizerComparison) -> ComparisonResult {
        // Render frames before action
        let mut all_differences = Vec::new();
        let mut max_difference = 0.0f32;
        let mut total_samples = 0;

        if self.frames_before_action > 0 {
            let result = synth.render_and_compare_frames(self.frames_before_action);
            all_differences.extend(result.differences);
            max_difference = max_difference.max(result.max_difference);
            total_samples += result.total_samples;
        }

        // Run action if present
        (self.action)(synth);

        // Render frames after action
        if self.frames_after_action > 0 {
            let result = synth.render_and_compare_frames(self.frames_after_action);
            all_differences.extend(result.differences);
            max_difference = max_difference.max(result.max_difference);
            total_samples += result.total_samples;
        }

        let passed = all_differences.is_empty();

        if synth.config.verbose {
            println!("\n=== Test Scenario: {} ===", self.name);
            synth.print_comparison_report(&all_differences, max_difference, total_samples);
        }

        ComparisonResult {
            total_samples,
            max_difference,
            differences: all_differences,
            passed,
        }
    }
}

/// Test scenario builder for common test patterns
pub struct TestScenario {
    pub name: String,
    pub setup: Box<dyn FnMut(&mut SynthesizerComparison)>,
    pub setup_frames: usize,
    pub actions: Vec<TestAction>,
}

impl TestScenario {
    pub fn new(
        setup: ChannelVoiceMessage,
        action: ChannelVoiceMessage,
        frames_before_action: usize,
        frames_after_action: usize,
    ) -> Self {
        Self {
            name: format!("Scenario -\nSetup: {setup:?}"),
            setup_frames: 0,
            setup: Box::new(move |synth| synth.process_midi_message(setup)),
            actions: vec![TestAction::new(
                format!("Action: {action:?}"),
                action,
                frames_before_action,
                frames_after_action,
            )],
        }
    }
    pub fn init(setup: Vec<ChannelVoiceMessage>, setup_frames: usize) -> Self {
        Self {
            name: format!("Scenario -\nSetup: {setup:#?}"),
            setup_frames,
            setup: Box::new(move |synth| {
                for message in setup.clone() {
                    synth.process_midi_message(message)
                }
            }),
            actions: Vec::new(),
        }
    }

    pub fn then(
        mut self,
        action: ChannelVoiceMessage,
        frames_before_action: usize,
        frames_after_action: usize,
    ) -> Self {
        self.actions.push(TestAction::new(
            format!("Addendum: {action:?}"),
            action,
            frames_before_action,
            frames_after_action,
        ));
        self
    }

    /// Run the test scenario and return the result
    pub fn run(&mut self, synth: &mut SynthesizerComparison) -> ComparisonResult {
        // Reset synthesizers
        synth.reset();

        // Run setup
        (self.setup)(synth);

        // Render frames before action
        let mut all_differences = Vec::new();
        let mut max_difference = 0.0f32;
        let mut total_samples = 0;

        if self.setup_frames > 0 {
            let result = synth.render_and_compare_frames(self.setup_frames);

            all_differences.extend(result.differences);
            max_difference = max_difference.max(result.max_difference);
            total_samples += result.total_samples;
        }

        for action in &mut self.actions {
            if action.frames_before_action > 0 {
                let result = synth.render_and_compare_frames(action.frames_before_action);

                all_differences.extend(result.differences);
                max_difference = max_difference.max(result.max_difference);
                total_samples += result.total_samples;
            }

            (action.action)(synth);

            if action.frames_after_action > 0 {
                let result = synth.render_and_compare_frames(action.frames_after_action);
                all_differences.extend(result.differences);
                max_difference = max_difference.max(result.max_difference);
                total_samples += result.total_samples;
            }
        }

        let passed = all_differences.is_empty();

        if synth.config.verbose {
            println!("\n=== Test Scenario: {} ===", self.name);
            synth.print_comparison_report(&all_differences, max_difference, total_samples);
        }

        ComparisonResult {
            total_samples,
            max_difference,
            differences: all_differences,
            passed,
        }
    }
}

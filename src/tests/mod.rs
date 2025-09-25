mod utils;
use midix::prelude::*;
use utils::*;

#[test]
fn test_basic_note_on_off() {
    let config = ComparisonConfig {
        epsilon: 5e-3, // Allow small floating point differences
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    let mut scenario = TestScenario::new(
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::note_on(
                Note::from_databyte(60).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::note_off(
                Note::from_databyte(60).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        10, // frames before note off
        10, // frames after note off
    );

    let result = scenario.run(&mut synth);

    assert!(
        result.passed,
        "Basic note on/off test failed with max difference: {:.9e}",
        result.max_difference
    );
}

#[test]
fn test_pitch_bend() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    // Test pitch bend up
    let mut scenario = TestScenario::new(
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::note_on(
                Note::from_databyte(60).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::PitchBend(PitchBend::new(0x00, 0x60).unwrap()), // bend up (12288)
        ),
        5,  // frames before bend
        10, // frames after bend
    );

    let result = scenario.run(&mut synth);
    assert!(
        result.passed,
        "Pitch bend up test failed with max difference: {:.9e}",
        result.max_difference
    );

    // Test pitch bend down
    let mut scenario = TestScenario::new(
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::note_on(
                Note::from_databyte(60).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::PitchBend(PitchBend::new(0x00, 0x20).unwrap()), // bend down (4096)
        ),
        5,  // frames before bend
        10, // frames after bend
    );

    let result = scenario.run(&mut synth);
    assert!(
        result.passed,
        "Pitch bend down test failed with max difference: {:.9e}",
        result.max_difference
    );
}

#[test]
fn test_volume_control() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    // Test volume change
    let mut scenario = TestScenario::new(
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::note_on(
                Note::from_databyte(60).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::ControlChange(Controller::VolumeCoarse(DataByte::new(64).unwrap())),
        ),
        5,  // frames before change
        10, // frames after change
    );

    let result = scenario.run(&mut synth);
    assert!(
        result.passed,
        "Volume control test failed with max difference: {:.9e}",
        result.max_difference
    );
}

#[test]
fn test_pan_control() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    // Test pan hard left
    let mut scenario = TestScenario::new(
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::note_on(
                Note::from_databyte(60).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::ControlChange(Controller::PanCoarse(DataByte::new(0).unwrap())),
        ),
        5,  // frames before change
        10, // frames after change
    );

    let result = scenario.run(&mut synth);
    assert!(
        result.passed,
        "Pan left test failed with max difference: {:.9e}",
        result.max_difference
    );

    // Test pan hard right
    let mut scenario = TestScenario::new(
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::note_on(
                Note::from_databyte(60).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::ControlChange(Controller::PanCoarse(DataByte::new(127).unwrap())),
        ),
        5,  // frames before change
        10, // frames after change
    );

    let result = scenario.run(&mut synth);
    assert!(
        result.passed,
        "Pan right test failed with max difference: {:.9e}",
        result.max_difference
    );
}

#[test]
fn test_sustain_pedal() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    // Custom scenario for sustain pedal
    synth.reset();

    // Note on
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_on(
            Note::from_databyte(60).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));

    // Render a few frames
    let _ = synth.render_and_compare_frames(5);

    // Press sustain pedal
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::ControlChange(Controller::damper_pedal(DataByte::new(127).unwrap())),
    ));

    // Note off (but should continue sounding due to sustain)
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_off(
            Note::from_databyte(60).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));

    // Render and check that sound continues
    let result = synth.render_and_compare_frames(5);
    assert!(
        result.passed,
        "Sustain pedal test (pedal on) failed with max difference: {:.9e}",
        result.max_difference
    );

    // Release sustain pedal
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::ControlChange(Controller::damper_pedal(DataByte::new(0).unwrap())),
    ));

    // Now the note should start releasing
    let result = synth.render_and_compare_frames(10);
    assert!(
        result.passed,
        "Sustain pedal test (pedal off) failed with max difference: {:.9e}",
        result.max_difference
    );
}

#[test]
fn test_modulation_wheel() {
    let config = ComparisonConfig {
        epsilon: 1e-8,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    let mut scenario = TestScenario::new(
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::note_on(
                Note::from_databyte(60).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::ControlChange(Controller::ModulationCoarse(DataByte::new(64).unwrap())),
        ),
        5,   // frames before change
        500, // frames after change
    );

    let result = scenario.run(&mut synth);
    assert!(
        result.passed,
        "Modulation wheel test failed with max difference: {:.9e}",
        result.max_difference
    );

    let mut then = scenario.then(
        ChannelVoiceMessage::new(
            Channel::One,
            VoiceEvent::note_off(Note::from_databyte(60).unwrap(), Velocity::MAX),
        ),
        0,
        5000,
    );

    let result = then.run(&mut synth);
    assert!(
        result.passed,
        "Modulation wheel test failed with max difference: {:.9e}",
        result.max_difference
    );
}

#[test]
fn detailed_modulation_wheel() {
    let config = ComparisonConfig {
        epsilon: 1e-8,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    let mut scenario = TestScenario::init(
        vec![
            VoiceEvent::program_change(Program::new_unchecked(0x4)).send_to_channel(Channel::One),
            VoiceEvent::note_on(note!(C, 3), Velocity::new_unchecked(100))
                .send_to_channel(Channel::One),
        ],
        5000,
    )
    .then(
        VoiceEvent::note_off(note!(C, 3), Velocity::MAX).send_to_channel(Channel::One),
        0,
        5000,
    );

    let result = scenario.run(&mut synth);
    assert!(
        result.passed,
        "Detailed odulation wheel test failed with max difference: {:.9e}",
        result.max_difference
    );
}
fn _test_multiple_notes() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    synth.reset();

    // Play a chord
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_on(
            Note::from_databyte(60).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_on(
            Note::from_databyte(64).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_on(
            Note::from_databyte(67).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));

    let result = synth.render_and_compare_frames(10);
    assert!(
        result.passed,
        "Chord test (note on) failed with max difference: {:.9e}",
        result.max_difference
    );

    // Release one note
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_off(
            Note::from_databyte(64).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));

    let result = synth.render_and_compare_frames(5);
    assert!(
        result.passed,
        "Chord test (partial release) failed with max difference: {:.9e}",
        result.max_difference
    );

    // Release remaining notes
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_off(
            Note::from_databyte(60).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_off(
            Note::from_databyte(67).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));

    let result = synth.render_and_compare_frames(10);
    assert!(
        result.passed,
        "Chord test (full release) failed with max difference: {:.9e}",
        result.max_difference
    );
}

#[test]
fn test_percussion_channel() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    // Channel 9 (index 9) is typically the percussion channel
    let mut scenario = TestScenario::new(
        ChannelVoiceMessage::new(
            Channel::Ten, // Channel 10 (percussion)
            VoiceEvent::note_on(
                Note::from_databyte(36).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        ChannelVoiceMessage::new(
            Channel::Ten,
            VoiceEvent::note_off(
                Note::from_databyte(36).unwrap(),
                Velocity::new(100).unwrap(),
            ),
        ),
        5, // frames before note off
        5, // frames after note off
    );

    let result = scenario.run(&mut synth);
    assert!(
        result.passed,
        "Percussion channel test failed with max difference: {:.9e}",
        result.max_difference
    );
}

#[test]
fn test_program_change() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    synth.reset();

    // Change to a different instrument
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::program_change(Program::new(1).unwrap()),
    ));

    // Play a note with the new instrument
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_on(
            Note::from_databyte(60).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));

    let result = synth.render_and_compare_frames(10);
    assert!(
        result.passed,
        "Program change test (note with new program) failed with max difference: {:.9e}",
        result.max_difference
    );

    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_off(
            Note::from_databyte(60).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));

    let result = synth.render_and_compare_frames(10);
    assert!(
        result.passed,
        "Program change test (release) failed with max difference: {:.9e}",
        result.max_difference
    );
}

fn _test_all_notes_off() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    synth.reset();

    // Play multiple notes
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_on(
            Note::from_databyte(60).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_on(
            Note::from_databyte(64).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_on(
            Note::from_databyte(67).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));

    let _ = synth.render_and_compare_frames(5);

    // All notes off controller
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::ControlChange(Controller::mute_all()),
    ));

    let result = synth.render_and_compare_frames(10);
    assert!(
        result.passed,
        "All notes off test failed with max difference: {:.9e}",
        result.max_difference
    );
}

fn _test_reset_all_controllers() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true,
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    synth.reset();

    // Set various controllers
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::ControlChange(Controller::ModulationCoarse(DataByte::new(127).unwrap())),
    ));
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::ControlChange(Controller::VolumeCoarse(DataByte::new(64).unwrap())),
    ));
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::ControlChange(Controller::PanCoarse(DataByte::new(0).unwrap())),
    ));
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::PitchBend(PitchBend::new(0x7F, 0x7F).unwrap()), // max pitch bend
    ));

    // Play a note
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::note_on(
            Note::from_databyte(60).unwrap(),
            Velocity::new(100).unwrap(),
        ),
    ));
    let _ = synth.render_and_compare_frames(5);

    // Reset all controllers
    synth.process_midi_message(ChannelVoiceMessage::new(
        Channel::One,
        VoiceEvent::ControlChange(Controller::reset_all()),
    ));

    let result = synth.render_and_compare_frames(10);
    assert!(
        result.passed,
        "Reset all controllers test failed with max difference: {:.9e}",
        result.max_difference
    );
}

#[test]
#[ignore] // This test can be slow
fn test_stress_many_notes() {
    let config = ComparisonConfig {
        epsilon: 5e-3,
        verbose: true, // Enable verbose output to debug the issue
        ..Default::default()
    };

    let mut synth = SynthesizerComparison::new("assets/soundfonts/8bitsf.sf2", config)
        .expect("Failed to create synthesizer comparison");

    synth.reset();

    // Map channel indices to Channel enum variants
    let channels = [
        Channel::One,
        Channel::Two,
        Channel::Three,
        Channel::Four,
        Channel::Five,
        Channel::Six,
        Channel::Seven,
        Channel::Eight,
    ];

    // Play many notes across different channels
    for channel in &channels {
        for note in (40..80).step_by(3) {
            synth.process_midi_message(ChannelVoiceMessage::new(
                *channel,
                VoiceEvent::note_on(
                    Note::from_databyte(note).unwrap(),
                    Velocity::new(80).unwrap(),
                ),
            ));
        }
    }

    let result = synth.render_and_compare_frames(20);
    assert!(
        result.passed,
        "Stress test (many notes) failed with max difference: {:.9e}",
        result.max_difference
    );

    // Release all notes
    for channel in &channels {
        for note in (40..80).step_by(3) {
            synth.process_midi_message(ChannelVoiceMessage::new(
                *channel,
                VoiceEvent::note_off(
                    Note::from_databyte(note).unwrap(),
                    Velocity::new(80).unwrap(),
                ),
            ));
        }
    }

    let result = synth.render_and_compare_frames(20);
    assert!(
        result.passed,
        "Stress test (release) failed with max difference: {:.9e}",
        result.max_difference
    );
}

use std::time::Instant;

use rpi::modules::{
    arduino::{format_dispense_command, parse_serial_response},
    audio::convert_i16_to_f32_mono,
    gemini::{VisorAnalysis, build_request_body},
};


#[test]
fn test_audio_normalization_and_downmix_latency() {
    // Simulate 1 second of stereo 48kHz audio (96,000 samples)
    let sample_count = 96_000;
    let stereo_samples: Vec<i16> = (0..sample_count)
        .map(|i| ((i % 65536) as i32 - 32768) as i16)
        .collect();

    let start = Instant::now();
    let normalized = convert_i16_to_f32_mono(&stereo_samples, 2);
    let duration = start.elapsed();

    assert_eq!(normalized.len(), 48_000);
    // Processing 1 full second of audio should take well under 5 milliseconds on modern CPU
    println!(
        "[LATENCY BENCHMARK] Audio downmixing (96k samples): {:?} (target < 5ms)",
        duration
    );
    assert!(
        duration.as_millis() < 50,
        "Audio normalization took too long: {:?}",
        duration
    );
}

#[test]
fn test_json_deserialization_latency() {
    // Note: 3rd item gauze_pad disabled for 2-item dispensing
    let raw_json = r#"{
        "can_help": true,
        "reasoning": "Minor abrasions and surface dirt present. Clean with alcohol pad and apply bandage.",
        "dispense": {
            "bandage": true,
            "alcohol_pad": true
        },
        "video_search_query": "how to clean scrape and bandage"
    }"#;

    let iterations = 10_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _analysis: VisorAnalysis = serde_json::from_str(raw_json).unwrap();
    }
    let total_duration = start.elapsed();
    let per_op_micros = total_duration.as_micros() as f64 / iterations as f64;

    println!(
        "[LATENCY BENCHMARK] JSON Deserialization: {:.2} µs/op across {} iterations",
        per_op_micros, iterations
    );
    assert!(
        per_op_micros < 100.0,
        "JSON deserialization exceeded threshold: {:.2} µs",
        per_op_micros
    );
}

#[test]
fn test_serial_packet_formatting_and_parsing_latency() {
    let iterations = 20_000;
    let start = Instant::now();
    for i in 0..iterations {
        let b = (i % 2) == 0;
        let a = (i % 3) == 0;
        // let g = (i % 5) == 0;

        let cmd = format_dispense_command(b, a/*, g*/);
        let _resp = parse_serial_response("ACK:DISP:1,0");
        let _ = cmd.len();
    }
    let total_duration = start.elapsed();
    let per_op_micros = total_duration.as_micros() as f64 / iterations as f64;

    println!(
        "[LATENCY BENCHMARK] Serial packet framing & parsing: {:.2} µs/op across {} iterations",
        per_op_micros, iterations
    );
    assert!(
        per_op_micros < 20.0,
        "Serial framing exceeded latency threshold: {:.2} µs",
        per_op_micros
    );
}

#[test]
fn test_request_payload_building_latency() {
    let base64_dummy = "A".repeat(400_000); // approx 300KB image encoded
    let iterations = 500;
    let start = Instant::now();
    for _ in 0..iterations {
        let body = build_request_body(&base64_dummy);
        let _ = body["model"].as_str();
    }
    let total_duration = start.elapsed();
    let per_op_micros = total_duration.as_micros() as f64 / iterations as f64;

    println!(
        "[LATENCY BENCHMARK] Gemini request payload generation (300KB image): {:.2} µs/op",
        per_op_micros
    );
    assert!(
        per_op_micros < 5000.0,
        "Payload building exceeded latency threshold: {:.2} µs",
        per_op_micros
    );
}

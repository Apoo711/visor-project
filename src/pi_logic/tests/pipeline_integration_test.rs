use rpi::modules::{
    arduino::{ArduinoResponse, format_dispense_command, parse_serial_response},
    audio::convert_i16_to_f32_mono,
    gemini::{DispenseItems, VisorAnalysis, build_request_body},
    input::ensure_parent_dir,
    youtube::{
        YouTubeSearchResponse, format_embed_url, parse_youtube_search_response, resolve_standby_url,
    },
};

#[test]
fn test_end_to_end_minor_injury_pipeline_flow() {
    // 1. Audio wake word stream preprocessing simulation
    let raw_stereo_audio: Vec<i16> = vec![
        1000, 1200, 2000, 2200, 3000, 3100, 4000, 4100, -1000, -1200, -2000, -2200,
    ];
    let normalized_mono = convert_i16_to_f32_mono(&raw_stereo_audio, 2);
    assert_eq!(normalized_mono.len(), 6);
    assert!(normalized_mono[0] > 0.0);
    assert!(normalized_mono[5] < 0.0);

    // 2. Camera Snapshot directory check & base64 payload construction
    let test_frame_dir = std::env::temp_dir().join("visor_pipeline_test");
    let test_frame_path = test_frame_dir.join("snapshot.jpg");
    assert!(ensure_parent_dir(test_frame_path.to_str().unwrap()).is_ok());

    let dummy_image_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46]; // JPEG header
    let base64_image =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &dummy_image_bytes);
    let request_payload = build_request_body(&base64_image);
    assert_eq!(request_payload["model"], "gemini-3.7-flash");

    // 3. AI Assessment Response Parsing
    // Note: 3rd item gauze_pad disabled for 2-item dispensing
    let simulated_ai_json = r#"{
        "can_help": true,
        "reasoning": "Minor skin laceration on left forefinger. Requires sanitization and sterile bandage.",
        "dispense": {
            "bandage": true,
            "alcohol_pad": true
        },
        "video_search_query": "how to properly dress a finger cut"
    }"#;
    let analysis: VisorAnalysis =
        serde_json::from_str(simulated_ai_json).expect("Pipeline analysis deserialization failed");
    assert!(analysis.can_help);
    assert!(analysis.dispense.bandage);
    assert!(analysis.dispense.alcohol_pad);
    // assert!(!analysis.dispense.gauze_pad); // [DISABLED: 3rd item]

    // 4. Arduino Dispense Command Generation & Simulated Handshake
    let serial_cmd = format_dispense_command(
        analysis.dispense.bandage,
        analysis.dispense.alcohol_pad,
        // analysis.dispense.gauze_pad,
    );
    assert_eq!(serial_cmd, "<DISP:1,1>\n");

    // Simulated Arduino ACK and Status sequence
    let simulated_ack = "ACK:DISP:1,1\r\n";
    let parsed_ack = parse_serial_response(simulated_ack);
    assert_eq!(
        parsed_ack,
        ArduinoResponse::AckDispense {
            bandage: true,
            alcohol: true,
            // gauze: false
        }
    );

    let simulated_status_bandage = "STATUS:DISPENSING_BANDAGE\n";
    assert_eq!(
        parse_serial_response(simulated_status_bandage),
        ArduinoResponse::StatusDispensing("BANDAGE".to_string())
    );

    let simulated_status_alcohol = "STATUS:DISPENSING_ALCOHOL\n";
    assert_eq!(
        parse_serial_response(simulated_status_alcohol),
        ArduinoResponse::StatusDispensing("ALCOHOL".to_string())
    );

    let simulated_status_complete = "STATUS:DISPENSE_COMPLETE\n";
    assert_eq!(
        parse_serial_response(simulated_status_complete),
        ArduinoResponse::StatusComplete
    );

    // 5. YouTube Search Result Resolution & Kiosk Embed Formatting
    let simulated_yt_response = r#"{
        "items": [
            {
                "id": { "videoId": "v1234567890" },
                "snippet": { "title": "First Aid 101: Dressing a Finger Cut" }
            }
        ]
    }"#;
    let yt_res: YouTubeSearchResponse = serde_json::from_str(simulated_yt_response).unwrap();
    let video_info = parse_youtube_search_response(yt_res);
    assert!(video_info.is_some());
    let (video_id, watch_url, title) = video_info.unwrap();
    assert_eq!(video_id, "v1234567890");
    assert_eq!(watch_url, "https://www.youtube.com/watch?v=v1234567890");
    assert_eq!(title, "First Aid 101: Dressing a Finger Cut");

    let embed_url = format_embed_url(&video_id);
    assert!(embed_url.contains("https://www.youtube-nocookie.com/embed/v1234567890"));
    assert!(embed_url.contains("autoplay=1"));

    // 6. Standby Screen Navigation Verification
    let standby_url = resolve_standby_url("non_existent_standby.html");
    assert!(standby_url.starts_with("data:text/html,"));

    // Cleanup
    let _ = std::fs::remove_dir_all(&test_frame_dir);
}

#[test]
fn test_end_to_end_emergency_hold_pipeline_flow() {
    // Simulated AI assessment for severe injury requiring emergency hospitalization
    // Note: 3rd item gauze_pad disabled for 2-item dispensing
    let simulated_emergency_json = r#"{
        "can_help": false,
        "reasoning": "Deep arterial laceration with severe hemorrhaging detected. Direct pressure required; emergency medical response must be summoned immediately.",
        "dispense": {
            "bandage": false,
            "alcohol_pad": false
        },
        "video_search_query": null
    }"#;

    let analysis: VisorAnalysis =
        serde_json::from_str(simulated_emergency_json).expect("Emergency triage deserialization failed");
    assert!(!analysis.can_help);
    assert_eq!(
        analysis.dispense,
        DispenseItems {
            bandage: false,
            alcohol_pad: false,
            // gauze_pad: false
        }
    );

    // Command generation should produce hold packet
    let serial_cmd = format_dispense_command(
        analysis.dispense.bandage,
        analysis.dispense.alcohol_pad,
        // analysis.dispense.gauze_pad,
    );
    assert_eq!(serial_cmd, "<DISP:0,0>\n");

    // Simulated Arduino Hold response
    let simulated_hold_response = "STATUS:HOLD_ALL\n";
    assert_eq!(
        parse_serial_response(simulated_hold_response),
        ArduinoResponse::StatusHoldAll
    );
}

#[test]
fn test_end_to_end_all_items_dispense_pipeline_flow() {
    // Note: 3rd item gauze_pad disabled for 2-item dispensing
    let simulated_all_supplies_json = r#"{
        "can_help": true,
        "reasoning": "Moderate scrape requiring alcohol prep and securing bandage.",
        "dispense": {
            "bandage": true,
            "alcohol_pad": true
        },
        "video_search_query": "how to clean scrape and apply bandage"
    }"#;

    let analysis: VisorAnalysis = serde_json::from_str(simulated_all_supplies_json).unwrap();
    assert!(analysis.can_help);
    assert!(analysis.dispense.bandage);
    assert!(analysis.dispense.alcohol_pad);
    // assert!(analysis.dispense.gauze_pad);

    let serial_cmd = format_dispense_command(
        analysis.dispense.bandage,
        analysis.dispense.alcohol_pad,
        // analysis.dispense.gauze_pad,
    );
    assert_eq!(serial_cmd, "<DISP:1,1>\n");

    let ack_resp = parse_serial_response("ACK:DISP:1,1");
    assert_eq!(
        ack_resp,
        ArduinoResponse::AckDispense {
            bandage: true,
            alcohol: true,
            // gauze: true
        }
    );
}

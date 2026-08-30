use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{error, info};
use rustpotter::{Rustpotter, RustpotterConfig, WakewordRef};
use tokio::sync::mpsc::{self, Receiver};

/// Downmixes an interleaved multi-channel floating-point audio slice into single-channel mono PCM samples.
///
/// If `channels` <= 1, returns a clone of the input slice.
/// For stereo or multi-channel audio, computes the arithmetic mean across all channel samples in each audio frame.
///
/// # Arguments
/// * `data` - Interleaved f32 audio samples.
/// * `channels` - Number of audio channels in the input buffer.
///
/// # Returns
/// * `Vec<f32>` - Mono f32 audio samples.
pub fn downmix_f32_to_mono(data: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    data.chunks_exact(channels as usize)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Converts interleaved 16-bit signed integer (i16) PCM samples to normalized mono floating-point (f32) samples in the `[-1.0, 1.0]` range.
///
/// Averages across channels for multi-channel inputs and scales values by 32768.0.
///
/// # Arguments
/// * `data` - Interleaved i16 audio samples from the input capture device.
/// * `channels` - Channel count (1 for mono, 2 for stereo, etc.).
///
/// # Returns
/// * `Vec<f32>` - Normalized single-channel mono samples.
pub fn convert_i16_to_f32_mono(data: &[i16], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        data.iter().map(|&s| s as f32 / 32768.0).collect()
    } else {
        data.chunks_exact(channels as usize)
            .map(|frame| {
                (frame.iter().map(|&s| s as f32).sum::<f32>() / channels as f32) / 32768.0
            })
            .collect()
    }
}

/// Real-time wake word detection engine listening on the system microphone using Rustpotter.
pub struct WakeWordDetector {
    _stream: Option<cpal::Stream>,
    rx: Arc<tokio::sync::Mutex<Receiver<()>>>,
}

impl WakeWordDetector {
    /// Initializes the microphone input stream via CPAL and sets up the Rustpotter keyword detector.
    ///
    /// Configures audio callback handlers to downmix incoming audio frames to mono f32
    /// and feed them to the keyword spotting model.
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn std::error::Error>>` - Active detector with running background audio stream,
    ///   or an error if microphone hardware initialization fails.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel(32);

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "No default audio input device (microphone) found")?;

        let device_desc = device
            .description()
            .map(|d| format!("{:?}", d))
            .unwrap_or_else(|_| "Default Microphone".to_string());
        info!("Audio input device selected: {}", device_desc);

        let default_config = device.default_input_config()?;
        let sample_rate = default_config.sample_rate();
        let channels = default_config.channels();
        info!(
            "Input audio stream config: {} Hz, {} channel(s), format: {:?}",
            sample_rate,
            channels,
            default_config.sample_format()
        );

        let mut config = RustpotterConfig::default();
        config.detector.avg_threshold = 0.5;
        config.detector.threshold = 0.55;

        let mut rustpotter = Rustpotter::new(&config)
            .map_err(|e| format!("Failed to create Rustpotter detector: {}", e))?;

        let wakeword_path = "assets/visor_help.rpw";
        if Path::new(wakeword_path).exists() {
            info!(
                "Loading custom Rustpotter wakeword model from '{}'",
                wakeword_path
            );
            rustpotter
                .add_wakeword_from_file("visor_help", wakeword_path)
                .map_err(|e| format!("Failed to load wakeword file: {}", e))?;
        } else {
            info!(
                "No '{}' model file found. Registering default 'visor_help' template.",
                wakeword_path
            );
            let wakeword_ref = WakewordRef {
                name: "visor_help".to_string(),
                avg_features: None,
                samples_features: HashMap::new(),
                threshold: Some(0.55),
                avg_threshold: Some(0.5),
                rms_level: 0.0,
                mfcc_size: 0,
            };
            let _ = rustpotter.add_wakeword_ref("visor_help", wakeword_ref);
        }

        let samples_per_frame = rustpotter.get_samples_per_frame();
        let rustpotter_arc = Arc::new(Mutex::new(rustpotter));
        let buffer_arc = Arc::new(Mutex::new(Vec::<f32>::new()));

        let rp_clone = rustpotter_arc.clone();
        let buf_clone = buffer_arc.clone();
        let tx_clone = tx.clone();

        let err_fn = |err| {
            error!("Audio input stream error: {}", err);
        };

        let stream_config: cpal::StreamConfig = default_config.into();

        let stream = match default_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono_samples = downmix_f32_to_mono(data, channels);

                    if let Ok(mut buf) = buf_clone.lock() {
                        buf.extend_from_slice(&mono_samples);
                        while buf.len() >= samples_per_frame {
                            let frame: Vec<f32> = buf.drain(..samples_per_frame).collect();
                            if let Ok(mut spotter) = rp_clone.lock() {
                                if let Some(detection) = spotter.process_samples(frame) {
                                    info!(
                                        "Wake word detected! Keyword: '{}' (Score: {:.2})",
                                        detection.name, detection.score
                                    );
                                    let _ = tx_clone.try_send(());
                                }
                            }
                        }
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mono_samples = convert_i16_to_f32_mono(data, channels);

                    if let Ok(mut buf) = buf_clone.lock() {
                        buf.extend_from_slice(&mono_samples);
                        while buf.len() >= samples_per_frame {
                            let frame: Vec<f32> = buf.drain(..samples_per_frame).collect();
                            if let Ok(mut spotter) = rp_clone.lock() {
                                if let Some(detection) = spotter.process_samples(frame) {
                                    info!(
                                        "Wake word detected! Keyword: '{}' (Score: {:.2})",
                                        detection.name, detection.score
                                    );
                                    let _ = tx_clone.try_send(());
                                }
                            }
                        }
                    }
                },
                err_fn,
                None,
            )?,
            sample_format => {
                return Err(format!("Unsupported audio sample format: {:?}", sample_format).into());
            }
        };

        stream.play()?;
        info!("Wake word audio listener active. Listening for 'VISOR help'...");

        Ok(Self {
            _stream: Some(stream),
            rx: Arc::new(tokio::sync::Mutex::new(rx)),
        })
    }

    /// Asynchronously blocks until the "VISOR help" wake word is triggered by the audio input stream.
    pub async fn wait_for_wake_word(&self) {
        let mut rx_guard = self.rx.lock().await;
        let _ = rx_guard.recv().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downmix_f32_mono() {
        let mono_in = vec![0.1, -0.2, 0.5, 0.8];
        let res = downmix_f32_to_mono(&mono_in, 1);
        assert_eq!(res, mono_in);
    }

    #[test]
    fn test_downmix_f32_stereo() {
        // [L1, R1, L2, R2]
        let stereo_in = vec![0.2, 0.4, -0.6, 0.2];
        let res = downmix_f32_to_mono(&stereo_in, 2);
        assert_eq!(res.len(), 2);
        assert!((res[0] - 0.3).abs() < 1e-6);
        assert!((res[1] - (-0.2)).abs() < 1e-6);
    }

    #[test]
    fn test_downmix_f32_quad_channel() {
        let quad_in = vec![0.1, 0.2, 0.3, 0.4];
        let res = downmix_f32_to_mono(&quad_in, 4);
        assert_eq!(res.len(), 1);
        assert!((res[0] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_convert_i16_to_f32_mono() {
        let i16_samples = vec![0, 16384, -16384, 32767, -32768];
        let res = convert_i16_to_f32_mono(&i16_samples, 1);
        assert_eq!(res.len(), 5);
        assert_eq!(res[0], 0.0);
        assert!((res[1] - 0.5).abs() < 1e-4);
        assert!((res[2] - (-0.5)).abs() < 1e-4);
        assert!((res[3] - 0.999969).abs() < 1e-4);
        assert_eq!(res[4], -1.0);
    }

    #[test]
    fn test_convert_i16_to_f32_stereo() {
        let stereo_i16 = vec![16384, 16384, -32768, 32767];
        let res = convert_i16_to_f32_mono(&stereo_i16, 2);
        assert_eq!(res.len(), 2);
        assert!((res[0] - 0.5).abs() < 1e-4);
        assert!((res[1] - (-0.000015)).abs() < 1e-4);
    }
}



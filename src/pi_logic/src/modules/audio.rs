use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{error, info};
use rustpotter::{Rustpotter, RustpotterConfig, WakewordRef};
use tokio::sync::mpsc::{self, Receiver};

pub struct WakeWordDetector {
    _stream: Option<cpal::Stream>,
    rx: Arc<tokio::sync::Mutex<Receiver<()>>>,
}

impl WakeWordDetector {
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
                    let mono_samples: Vec<f32> = if channels > 1 {
                        data.chunks_exact(channels as usize)
                            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                            .collect()
                    } else {
                        data.to_vec()
                    };

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
                    let mono_samples: Vec<f32> = if channels > 1 {
                        data.chunks_exact(channels as usize)
                            .map(|frame| {
                                (frame.iter().map(|&s| s as f32).sum::<f32>() / channels as f32)
                                    / 32768.0
                            })
                            .collect()
                    } else {
                        data.iter().map(|&s| s as f32 / 32768.0).collect()
                    };

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

    pub async fn wait_for_wake_word(&self) {
        let mut rx_guard = self.rx.lock().await;
        let _ = rx_guard.recv().await;
    }
}

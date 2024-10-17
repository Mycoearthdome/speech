extern crate cpal;
extern crate vosk;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use vosk::{Model, Recognizer};

const SAMPLE_RATE: f32 = 44100.0;
const MAX_AMPLITUDE: f32 = 32767.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the default host
    let host = cpal::default_host();

    // Get the default input device
    let input_device = host
        .default_input_device()
        .ok_or("No input device available")?;

    // Load the Vosk model
    let model =
        Model::new("vosk-model-en-us-0.42-gigaspeech").unwrap();

    // Create the recognizer
    let recognizer = Recognizer::new(&model, SAMPLE_RATE).ok_or("Failed to create recognizer")?;
    let recognizer = Arc::new(Mutex::new(recognizer));

    // Get the input configuration for the default input device
    let input_config = input_device
        .default_input_config()
        .map_err(|_| "Failed to get default input config")?
        .config();

    // Create the input stream once
    let stream = input_device.build_input_stream(
        &input_config,
        {
            let recognizer = Arc::clone(&recognizer);
            move |data: &[f32], _| {
                // Convert the f32 samples to i16 samples
                let data: Vec<i16> = data.iter().map(|x| (*x * MAX_AMPLITUDE) as i16).collect();

                // Feed the samples into the recognizer
                let mut recognizer_lock = recognizer.lock().unwrap();
                match recognizer_lock.accept_waveform(&data) {
                    vosk::DecodingState::Finalized => {
                        let words = recognizer_lock.final_result().single().unwrap().text;
                        let trimmed_words = words.trim_matches('"').to_string();
                        if trimmed_words.len() > 0 {
                            if !trimmed_words.contains("shh") {
                                print!("{} ", trimmed_words);
                            }
                        }
                    }
                    vosk::DecodingState::Running => {
                        let words = recognizer_lock.partial_result().partial;
                        let trimmed_words = words.trim_matches('"').to_string();
                        if trimmed_words.len() > 0 {
                            if !trimmed_words.contains("shh") {
                                print!("{} ", trimmed_words);
                            }
                        }
                    }
                    _ => {}
                }
            }
        },
        |err| {
            eprintln!("Error occurred on stream: {}", err);
        },
        None,
    )?;

    // Play the stream
    stream.play()?;

    println!("Stream is now active. Press Ctrl+C to stop...");

    // Keep the main thread alive while the stream is running
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

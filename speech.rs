extern crate cpal;
extern crate vosk;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use vosk::{Model, Recognizer};

fn main() {
    // Get the default host
    let host = cpal::default_host();

    // Get the default input device
    let input_device = host
        .default_input_device()
        .expect("No input device available");

    // Load the Vosk model
    let model = Model::new("vosk-model-en-us-0.42-gigaspeech")
        .expect("Failed to load model");

    // Create the recognizer
    let recognizer = Recognizer::new(&model, 16000.0).expect("Failed to create recognizer");
    let recognizer = Arc::new(Mutex::new(recognizer));

    // Get the input configuration for the default input device
    let input_config = input_device
        .default_input_config()
        .expect("Failed to get default input config")
        .config();

    // Create the input stream once
    let stream = input_device
        .build_input_stream(
            &input_config,
            {
                let recognizer = Arc::clone(&recognizer);
                move |data: &[f32], _| {
                    // Convert f32 samples to i16
                    let samples: Vec<i16> = data.iter().map(|&x| (x * 32767.0) as i16).collect();

                    // Feed the samples into the recognizer
                    let mut recognizer_lock = recognizer.lock().unwrap();
                    //recognizer_lock.set_max_alternatives(5);
                    match recognizer_lock.accept_waveform(&samples) {
                        vosk::DecodingState::Finalized => {
                            print!(
                                "{:#?}",
                                recognizer_lock.final_result().single().unwrap().text
                            );
                        }
                        _ => {}
                    }
                }
            },
            |err| {
                eprintln!("Error occurred on stream: {}", err);
            },
            None,
        )
        .expect("Failed to build input stream");

    // Play the stream
    stream.play().expect("Failed to start stream");

    println!("Stream is now active. Press Ctrl+C to stop...");

    // Keep the main thread alive while the stream is running
    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

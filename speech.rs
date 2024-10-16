extern crate cpal;
extern crate vosk;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use vosk::{Model, Recognizer};

fn main() {
    // Get the default host
    let host = cpal::default_host();

    // Get the default input device
    let input_device = host
        .default_input_device()
        .expect("No input device available");

    // Load the Vosk model
    let model = Model::new("/home/jordan/Documents/speech/small/vosk-model-small-en-us-0.15")
        .expect("Failed to load model");

    // Create the recognizer
    let recognizer = Recognizer::new(&model, 16000.0).expect("Failed to create recognizer");
    let recognizer = Arc::new(Mutex::new(recognizer));

    // Create the input stream
    let stream_config = input_device
        .default_input_config()
        .expect("Failed to get default input config")
        .config();
    let mut playing = false;
    loop {
        let stream = input_device
            .build_input_stream(
                &stream_config,
                {
                    let recognizer = Arc::clone(&recognizer);
                    move |data: &[f32], _| {
                        // Convert f32 samples to i16
                        let samples: Vec<i16> =
                            data.iter().map(|&x| (x * 32767.0) as i16).collect();
                        match recognizer.lock().unwrap().accept_waveform(&samples) {
                            vosk::DecodingState::Finalized => {
                                println!("{:#?}", recognizer.lock().unwrap().result());
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

        if !playing {
            // Start the stream
            stream.play().expect("Failed to start stream");
            playing = true;
        }
    }

    // Keep the main thread alive to listen for input
}

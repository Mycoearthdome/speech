extern crate cpal;
extern crate vosk;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat::I16;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
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
        Model::new("/home/jordan/Documents/speech/vosk-model-en-us-0.42-gigaspeech").unwrap();

    // Create the recognizer
    let recognizer = Recognizer::new(&model, SAMPLE_RATE).ok_or("Failed to create recognizer")?;
    let recognizer = Arc::new(Mutex::new(recognizer));

    // Get the input configuration for the default input device
    let mut input_config = input_device
        .default_input_config()
        .map_err(|_| "Failed to get default input config")?
        .config();

    // Ensure the sample rate is set to 16000
    input_config.sample_rate.0 = SAMPLE_RATE as u32;
    input_config.channels = 1;
    input_config.buffer_size = cpal::BufferSize::Fixed(44100 as u32);

    // Create the input stream once
    let stream = input_device.build_input_stream_raw(
        &input_config,
        I16,
        {
            let recognizer = Arc::clone(&recognizer);
            move |data: &cpal::Data, _: &cpal::InputCallbackInfo| {
                // Convert the data to a slice of i16 samples
                let data: &[i16] = match data.sample_format() {
                    cpal::SampleFormat::I16 => data.as_slice::<i16>().unwrap(),
                    _ => panic!("Unexpected data format"),
                };

                // Feed the samples into the recognizer
                let mut recognizer_lock = recognizer.lock().unwrap();

                match recognizer_lock.accept_waveform(data) {
                    vosk::DecodingState::Finalized => {
                        let words = recognizer_lock.final_result().single().unwrap();
                        let trimmed_words = words.text.trim_matches('"').to_string();
                        if !trimmed_words.is_empty() {
                            print!("{} ", trimmed_words);
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
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

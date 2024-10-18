extern crate cpal;
extern crate libretranslate;
extern crate tokio;
extern crate vosk;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat::I16;
use libretranslate::{Language, TranslateError, Translation};
use std::time::Duration;
use std::{
    sync::{mpsc, Arc, Mutex},
    //time::Duration,
};
use vosk::{Model, Recognizer};

const SAMPLE_RATE: f32 = 44100.0;
//const MAX_AMPLITUDE: f32 = 32767.0;

async fn translate(trimmed_words: String) -> Result<Translation, TranslateError> {
    libretranslate::translate_url(
        Language::English,
        Language::French,
        trimmed_words,
        "http://192.168.0.226:5000/".to_string(),
        None,
    )
    .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    input_config.buffer_size = cpal::BufferSize::Default;

    // Create a channel to send trimmed_results to an async context
    let (tx, rx) = mpsc::channel();

    // Create the input stream once
    let stream = input_device.build_input_stream_raw(
        &input_config,
        I16,
        {
            let recognizer = Arc::clone(&recognizer);
            let tx = tx.clone();
            move |data: &cpal::Data, _: &cpal::InputCallbackInfo| {
                let mut trimmed_results = String::new();
                let recognizer = Arc::clone(&recognizer);
                // Convert the data to a Vec<i16> before moving it into the async block
                let data: Vec<i16> = match data.sample_format() {
                    cpal::SampleFormat::I16 => data.as_slice::<i16>().unwrap().to_vec(),
                    _ => panic!("Unexpected data format"),
                };
                let mut recognizer_lock = recognizer.lock().unwrap();
                if recognizer_lock.accept_waveform(&data) == vosk::DecodingState::Finalized {
                    trimmed_results = recognizer_lock
                        .result()
                        .single()
                        .unwrap()
                        .text
                        .trim_matches('"')
                        .to_string();
                }
                if !trimmed_results.is_empty() {
                    let _ = tx.send(trimmed_results);
                }
            }
        },
        |err| {
            eprintln!("Error occurred on stream: {}", err);
        },
        Some(Duration::from_secs(3)), //detects silences.
    )?;

    // Spawn a task to receive trimmed_results and process them
    tokio::spawn(async move {
        while let Ok(trimmed_results) = rx.recv() {
            //print!("{} ", trimmed_results);
            if !trimmed_results.is_empty() {
                match translate(trimmed_results).await {
                    Ok(translation) => {
                        let translated_text = translation.output;
                        print!("{} ", translated_text);
                    }
                    Err(e) => {
                        eprintln!("Error occurred during translation: {}", e);
                    }
                }
            }
        }
    });

    // Play the stream
    stream.play()?;

    println!("Stream is now active. Press Ctrl+C to stop...");

    // Keep the main thread alive while the stream is running
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

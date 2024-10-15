extern crate portaudio;
extern crate vosk;

use portaudio::PortAudio;
use vosk::{Model, Recognizer};

fn main() {
    // Load the Vosk model
    let model = Model::new("YOUR MODEL HERE")
        .ok_or("Failed to load model")
        .unwrap();
    let recognizer = Recognizer::new(&model, 16000.0)
        .ok_or("Failed to create recognizer")
        .unwrap();
    let recognizer = std::sync::Arc::new(std::sync::Mutex::new(recognizer));
    let pa = PortAudio::new().unwrap();
    let input_params =
        portaudio::StreamParameters::<f32>::new(pa.default_input_device().unwrap(), 1, true, 0.0);
    let output_params =
        portaudio::StreamParameters::<f32>::new(pa.default_output_device().unwrap(), 2, true, 0.0);
    let settings = portaudio::DuplexStreamSettings::new(input_params, output_params, 16000.0, 1024);

    let mut stream = pa
        .open_non_blocking_stream(settings, {
            let recognizer = recognizer.clone();
            move |args| {
                let mut recognizer = recognizer.lock().unwrap();
                // Convert the in_buffer from f32 to i16
                let in_buffer_i16: Vec<i16> = args
                    .in_buffer
                    .iter()
                    .map(|x| (*x * 32767.0) as i16)
                    .collect();
                recognizer.accept_waveform(&in_buffer_i16);
                println!("{:#?}", recognizer.final_result());
                portaudio::Continue
            }
        })
        .unwrap();

    // Start the stream
    stream.start().unwrap();

    // Wait for the stream to finish
    stream.stop().unwrap();
    stream.close().unwrap();
}

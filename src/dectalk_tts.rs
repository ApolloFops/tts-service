use std::{io::Cursor, sync::Arc};

use dectalk;
use hound::{SampleFormat, WavSpec, WavWriter};
use reqwest::header::HeaderValue;
use tokio::sync::Mutex;

use crate::Result;

pub const DATA_BUFFER_SIZE: usize = 4096;
pub const INDEX_BUFFER_SIZE: usize = 128;

pub fn setup_tts() -> dectalk::TTSHandle {
    let mut handle: dectalk::TTSHandle = dectalk::TTSHandle::new();

    handle.startup(0, 0).expect("Failed to start DECTalk");
    handle
        .open_in_memory(dectalk::DtTTSFormat::WaveFormat1M16)
        .expect("Failed to open DECTalk in memory");
    handle
        .create_buffer(DATA_BUFFER_SIZE, INDEX_BUFFER_SIZE)
        .expect("Failed to create DECTalk buffer");

    return handle;
}

pub async fn get_tts(
    text: &str,
    voice: &str,
    handle_mutex: Arc<Mutex<dectalk::TTSHandle>>,
) -> Result<(bytes::Bytes, Option<HeaderValue>)> {
    if !check_voice(voice) {
        anyhow::bail!("Invalid voice: {voice}");
    }

    // Get a lock on the TTS handle
    let mut handle = handle_mutex.lock().await;

    // Get the raw data
    let raw_data = handle
        .speak(text, dectalk::DtTTSFlags::Force)
        .expect("Failed to queue speech")
        .await;

    // Set up the WAV file
    let spec = WavSpec {
        channels: 1,
        sample_rate: 11025,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut cursor, spec).unwrap();

    for chunk in raw_data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        writer.write_sample(sample).unwrap();
    }

    writer.finalize().unwrap();
    let wav_bytes = cursor.into_inner();

    // Return the WAV file and header
    Ok((
        bytes::Bytes::from(wav_bytes),
        Some(HeaderValue::from_static("audio/wav")),
    ))
}

pub fn get_voices() -> Vec<String> {
    vec![]
}

pub fn check_voice(voice: &str) -> bool {
    // TODO: Add actual logic for voices
    return true;
}

use std::fs::File;
use std::path::Path;

use symphonia::core::audio::GenericAudioBufferRef;
use symphonia::core::codecs::audio::{AudioDecoder as SymphoniaAudioDecoder, AudioDecoderOptions};
use symphonia::core::codecs::CodecParameters;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::Time;
use symphonia::default::{get_codecs, get_probe};

pub struct AudioDecoder {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn SymphoniaAudioDecoder>,
    track_id: u32,
}

impl AudioDecoder {
    pub fn new(path: &Path) -> Result<Self, symphonia::core::errors::Error> {
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let hint = Hint::new();
        let reader = get_probe().probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())?;

        let (track_id, decoder) = {
            let track = reader
                .default_track(TrackType::Audio)
                .ok_or(symphonia::core::errors::Error::Unsupported(
                    "no audio track found",
                ))?;

            let audio_params = match &track.codec_params {
                Some(CodecParameters::Audio(params)) => params,
                _ => {
                    return Err(symphonia::core::errors::Error::Unsupported(
                        "no audio codec parameters",
                    ))
                }
            };

            let dec = get_codecs().make_audio_decoder(audio_params, &AudioDecoderOptions::default())?;
            (track.id, dec)
        };

        Ok(AudioDecoder {
            reader,
            decoder,
            track_id,
        })
    }

    pub fn decode(&mut self) -> Result<GenericAudioBufferRef, symphonia::core::errors::Error> {
        loop {
            let packet = match self.reader.next_packet()? {
                Some(p) => p,
                None => {
                    return Err(symphonia::core::errors::Error::IoError(
                        std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "end of stream"),
                    ))
                }
            };

            if packet.track_id != self.track_id {
                continue;
            }

            let decoded = self.decoder.decode(&packet)?;
            return Ok(decoded);
        }
    }

    pub fn seek(&mut self, time: u64) -> Result<(), symphonia::core::errors::Error> {
        let seek_to = SeekTo::Time {
            time: Time::from_millis_u64(time),
            track_id: Some(self.track_id),
        };
        self.reader.seek(SeekMode::Accurate, seek_to)?;
        self.decoder.reset();
        Ok(())
    }

    pub fn duration(&self) -> u64 {
        let sample_rate = self.sample_rate() as u64;
        if sample_rate == 0 {
            return 0;
        }
        self.reader
            .tracks()
            .iter()
            .find(|t| t.id == self.track_id)
            .and_then(|t| t.duration.as_ref())
            .map(|d| d.get() * 1000 / sample_rate)
            .unwrap_or(0)
    }

    pub fn channels(&self) -> u16 {
        self.decoder
            .codec_params()
            .channels
            .as_ref()
            .map(|c| c.count() as u16)
            .unwrap_or(2)
    }

    pub fn sample_rate(&self) -> u32 {
        self.decoder.codec_params().sample_rate.unwrap_or(44100)
    }
}

pub fn convert_audio_buffer(buf: GenericAudioBufferRef) -> Vec<f32> {
    let mut samples = Vec::new();
    buf.copy_to_vec_interleaved(&mut samples);
    samples
}

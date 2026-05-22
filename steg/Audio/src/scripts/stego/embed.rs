use std::path::Path;

use hound::{SampleFormat, WavReader, WavWriter};

use crate::scripts::error::StegoError;

use super::header;

pub fn embed_message<I, O>(input: I, output: O, message: &str) -> Result<(), StegoError>
where
    I: AsRef<Path>,
    O: AsRef<Path>,
{
    embed_bytes(input, output, message.as_bytes())
}

pub fn embed_bytes<I, O>(input: I, output: O, payload: &[u8]) -> Result<(), StegoError>
where
    I: AsRef<Path>,
    O: AsRef<Path>,
{
    let mut reader = WavReader::open(input)?;
    let spec = reader.spec();
    validate_spec(spec)?;

    let mut samples = reader.samples::<i16>().collect::<Result<Vec<_>, _>>()?;
    embed(&mut samples, payload)?;

    let mut writer = WavWriter::create(output, spec)?;
    for sample in samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;

    Ok(())
}

pub fn embed(samples: &mut [i16], payload: &[u8]) -> Result<(), StegoError> {
    if payload.len() > max_capacity(samples.len()) {
        let needed = (header::header_len_bytes() + payload.len()) * 8;
        return Err(StegoError::PayloadTooLarge {
            needed,
            available: samples.len(),
        });
    }

    let packed = header::pack(payload)?;

    for (sample, bit) in samples.iter_mut().zip(header::bytes_to_bits(&packed)) {
        let bit = i16::from(bit);
        *sample = (*sample & !1) | bit;
    }

    Ok(())
}

pub fn max_capacity(sample_count: usize) -> usize {
    let byte_capacity = sample_count / 8;
    byte_capacity.saturating_sub(header::header_len_bytes())
}

fn validate_spec(spec: hound::WavSpec) -> Result<(), StegoError> {
    if spec.sample_format != SampleFormat::Int || spec.bits_per_sample != 16 {
        return Err(StegoError::UnsupportedFormat(format!(
            "expected 16-bit PCM, got {:?} with {} bits per sample",
            spec.sample_format, spec.bits_per_sample
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use hound::WavSpec;

    use super::*;
    use crate::scripts::stego::extract::extract_message;

    fn make_pcm(size: usize) -> Vec<i16> {
        vec![0xAB; size]
    }

    #[test]
    fn test_embed_succeeds_with_sufficient_pcm() {
        let mut pcm = make_pcm(44_100 * 2);

        assert!(embed(&mut pcm, b"hello world").is_ok());
    }

    #[test]
    fn test_embed_fails_when_payload_too_large() {
        let mut pcm = make_pcm(200);
        let big_payload = vec![0u8; 10_000];

        assert!(embed(&mut pcm, &big_payload).is_err());
    }

    #[test]
    fn test_max_capacity_is_nonzero_for_reasonable_pcm() {
        assert!(max_capacity(44_100 * 2) > 0);
    }

    #[test]
    fn test_max_capacity_zero_for_tiny_pcm() {
        assert_eq!(max_capacity(10), 0);
    }

    #[test]
    fn embeds_and_extracts_message_from_wav() {
        let base = std::env::temp_dir().join(format!(
            "audio_steg_{}_{}",
            std::process::id(),
            "round_trip"
        ));
        let input = base.with_extension("input.wav");
        let output = base.with_extension("output.wav");
        let message = "hidden in wav";

        write_test_wav(&input);
        embed_message(&input, &output, message).unwrap();

        assert_eq!(extract_message(&output).unwrap(), message);

        let _ = fs::remove_file(input);
        let _ = fs::remove_file(output);
    }

    fn write_test_wav(path: &Path) {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(path, spec).unwrap();

        for i in 0..4096 {
            writer.write_sample((i % 256) as i16).unwrap();
        }

        writer.finalize().unwrap();
    }
}

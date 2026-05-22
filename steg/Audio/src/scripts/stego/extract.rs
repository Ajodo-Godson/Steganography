use std::path::Path;

use hound::{SampleFormat, WavReader};

use crate::scripts::error::StegoError;

use super::header;

pub fn extract_message<I>(input: I) -> Result<String, StegoError>
where
    I: AsRef<Path>,
{
    let bytes = extract_bytes(input)?;
    Ok(String::from_utf8(bytes)?)
}

pub fn extract_bytes<I>(input: I) -> Result<Vec<u8>, StegoError>
where
    I: AsRef<Path>,
{
    let mut reader = WavReader::open(input)?; // Opens the wav
    let spec = reader.spec(); // Reads the spec
    validate_spec(spec)?; // validates that it's a 16-bit PCM 

    let samples = reader.samples::<i16>().collect::<Result<Vec<_>, _>>()?; // Reads the samples as i16
    let header_bits = read_bits(&samples, header::header_len_bits())?; // Reads only enough bits for the 9-byte header
    let header_bytes = header::bits_to_bytes(&header_bits); // converts the bits back into bytes
    let payload_len = header::payload_len_from_header(&header_bytes)?; // reads the payload length from the header
    let total_bits = header::header_len_bits() + payload_len * 8; // calculates the full number of bits needed
    let bits = read_bits(&samples, total_bits)?; // reads header and payload bits
    let packed = header::bits_to_bytes(&bits); // converts the bits back into bytes

 
    Ok(header::unpack(&packed)?.to_vec()) // unpacks and retursn only the payload 
}

fn read_bits(samples: &[i16], len: usize) -> Result<Vec<bool>, StegoError> {
    if len > samples.len() {
        return Err(StegoError::PayloadTooLarge {
            needed: len,
            available: samples.len(),
        });
    }

    Ok(samples
        .iter()
        .take(len)
        .map(|sample| (sample & 1) == 1)
        .collect())
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

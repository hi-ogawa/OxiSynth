mod utils;

pub mod hydra;
pub mod info;
pub mod sample_data;

use crate::error::ParseError;
use std::io::{Read, Seek};

pub use hydra::*;
pub use info::*;
pub use sample_data::*;

#[derive(Debug)]
pub struct SFData {
    pub info: Info,
    pub sample_data: SampleData,
    pub hydra: Hydra,
}

impl SFData {
    pub fn load<F: Read + Seek>(file: &mut F) -> Result<Self, ParseError> {
        let sfbk = riff::Chunk::read(file, 0)?;
        if sfbk.id() != riff::RIFF_ID {
            return Err(ParseError::OtherError(
                "expected 'RIFF' chunk id".to_string(),
            ));
        }
        let chunk_type = sfbk.read_type(file)?;
        if chunk_type.value != "sfbk".as_bytes() {
            return Err(ParseError::OtherError(
                "expected 'sfbk' chunk type".to_string(),
            ));
        }

        let chunks: Vec<_> = sfbk.iter(file).collect();

        let mut info = None;
        let mut sample_data = None;
        let mut hydra = None;

        for ch in chunks.into_iter() {
            if ch.id() != riff::LIST_ID {
                return Err(ParseError::OtherError(
                    "expected 'LIST' chunk id".to_string(),
                ));
            }
            let ty = ch.read_type(file)?;
            match ty.as_str() {
                "INFO" => {
                    info = Some(Info::read(&ch, file)?);
                }
                "sdta" => {
                    sample_data = Some(SampleData::read(&ch, file)?);
                }
                "pdta" => {
                    hydra = Some(Hydra::read(&ch, file)?);
                }
                _ => {
                    return Err(ParseError::UnexpectedMemeberOfRoot(ch));
                }
            }
        }

        Ok(SFData {
            info: info.unwrap(),
            sample_data: sample_data.unwrap(),
            hydra: hydra.unwrap(),
        })
    }
}

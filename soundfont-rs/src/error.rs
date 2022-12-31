use std::array::TryFromSliceError;
use std::str::Utf8Error;

use riff::Chunk;

#[derive(Debug)]
pub enum ParseError {
    OtherError(String),
    IOError(std::io::Error),
    StringError(Utf8Error),
    NumSliceError(TryFromSliceError),

    InvalidBagChunkSize(u32),
    InvalidGeneratorChunkSize(u32),
    InvalidInstrumentChunkSize(u32),
    InvalidModulatorChunkSize(u32),
    InvalidPresetChunkSize(u32),
    InvalidSampleChunkSize(u32),

    UnknownGeneratorType(u16),
    UnknownSampleType(u16),
    UnknownModulatorTransform(u16),

    UnexpectedMemeberOfRoot(Chunk),
    UnexpectedMemeberOfHydra(Chunk),
    UnexpectedMemeberOfInfo(Chunk),
    UnexpectedMemeberOfSampleData(Chunk),
}

impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        Self::StringError(err)
    }
}

impl From<TryFromSliceError> for ParseError {
    fn from(err: TryFromSliceError) -> Self {
        Self::NumSliceError(err)
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}

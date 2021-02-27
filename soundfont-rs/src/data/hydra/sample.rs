use super::super::utils::Reader;
use riff::Chunk;

#[derive(Debug, Clone)]
pub struct SFSampleHeader {
    pub name: String,

    pub start: u32,
    pub end: u32,
    pub loop_start: u32,
    pub loop_end: u32,
    pub sample_rate: u32,

    pub origpitch: u8,
    pub pitchadj: i8,
    pub sample_link: u16,
    pub sample_type: SFSampleLink,
}

impl SFSampleHeader {
    pub fn read(reader: &mut Reader) -> Self {
        let name: String = reader.read_string(20);
        // 20

        let start: u32 = reader.read_u32();
        // 24
        let end: u32 = reader.read_u32();
        // 28
        let loop_start: u32 = reader.read_u32();
        // 32
        let loop_end: u32 = reader.read_u32();
        // 36

        let sample_rate: u32 = reader.read_u32();
        // 40

        let origpitch: u8 = reader.read_u8();
        // 41
        let pitchadj: i8 = reader.read_i8();
        // 42
        let sample_link: u16 = reader.read_u16();
        // 44
        let sample_type: u16 = reader.read_u16();

        let sample_type = match sample_type {
            0 => SFSampleLink::None,
            1 => SFSampleLink::MonoSample,
            2 => SFSampleLink::RightSample,
            4 => SFSampleLink::LeftSample,
            8 => SFSampleLink::LinkedSample,
            0x8001 => SFSampleLink::RomMonoSample,
            0x8002 => SFSampleLink::RomRightSample,
            0x8004 => SFSampleLink::RomLeftSample,
            0x8008 => SFSampleLink::RomLinkedSample,
            0x11 => SFSampleLink::VorbisMonoSample,
            0x12 => SFSampleLink::VorbisRightSample,
            0x14 => SFSampleLink::VorbisLeftSample,
            0x18 => SFSampleLink::VorbisLinkedSample,

            v => {
                panic!("Unknown SFSampleLink, {:?}", v);
            }
        };

        Self {
            name,
            start,
            end,
            loop_start,
            loop_end,
            sample_rate,
            origpitch,
            pitchadj,
            sample_link,
            sample_type,
        }
    }

    pub fn read_all(phdr: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
        assert_eq!(phdr.id().as_str(), "shdr");

        let size = phdr.len();
        if size % 46 != 0 || size == 0 {
            panic!("Instrument header chunk size is invalid");
        }

        let amount = size / 46;

        let data = phdr.read_contents(file).unwrap();
        let mut reader = Reader::new(data);

        (0..amount).map(|_| Self::read(&mut reader)).collect()
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum SFSampleLink {
    None = 0,

    /// Value used for mono samples
    MonoSample = 0x1,
    /// Value used for right samples of a stereo pair    
    RightSample = 0x2,
    /// Value used for left samples of a stereo pair
    LeftSample = 0x4,
    /// Value used for linked sample
    LinkedSample = 0x8,

    RomMonoSample = 0x8001,
    RomRightSample = 0x8002,
    RomLeftSample = 0x8004,
    RomLinkedSample = 0x8008,

    VorbisMonoSample = 0x11,
    VorbisRightSample = 0x12,
    VorbisLeftSample = 0x14,
    VorbisLinkedSample = 0x18,
}

impl SFSampleLink {
    pub fn is_mono(&self) -> bool {
        match self {
            Self::MonoSample | Self::RomMonoSample | Self::VorbisMonoSample => true,
            _ => false,
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            Self::RightSample | Self::RomRightSample | Self::VorbisRightSample => true,
            _ => false,
        }
    }

    pub fn is_left(&self) -> bool {
        match self {
            Self::LeftSample | Self::RomLeftSample | Self::VorbisLeftSample => true,
            _ => false,
        }
    }

    pub fn is_linked(&self) -> bool {
        match self {
            Self::LinkedSample | Self::RomLinkedSample | Self::VorbisLinkedSample => true,
            _ => false,
        }
    }
}
//!.panic
//! Provides utilities like [``decode_file``] for decoding NEXRAD data.
//!

use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::mem::size_of;
use std::path::Path;

use crate::decompress::decompress_file;
use crate::file_metadata::is_compressed;
use crate::model::{
    DataBlockHeader, DataBlockProduct, DataMoment, ElevationData, GenericData, Message31,
    Message31Header, MessageHeader, RadialData, VolumeData, VolumeHeaderRecord,
};
use anyhow::Result;

/// A decoded NEXRAD WSR-88D data file including sweep data.
pub struct DataFile {
    volume_header: VolumeHeaderRecord,
    elevation_scans: BTreeMap<u8, Vec<Message31>>,
}

impl DataFile {
    /// Load a nexrad file from a file path, decoding if necessary
    ///
    /// # Errors
    /// Returns an error if the file is not a valid NEXRAD file.
    pub fn new(file_path: &Path) -> Result<Self> {
        let data = std::fs::read(file_path)?;

        if is_compressed(&data) {
            let decompressed = decompress_file(&data)?;
            Self::from_vec(decompressed)
        } else {
            Self::from_vec(data)
        }
    }

    /// Load a nexrad file from byte slice.
    ///
    /// # Errors
    /// Returns an error if the file is not a valid NEXRAD file.
    pub fn from_slice(data: &[u8]) -> Result<Self> {
        Self::from_vec(data.to_vec())
    }

    /// Given an uncompressed data file, decodes it and returns the decoded structure.
    ///
    /// # Errors
    /// Returns an error if the file is not a valid NEXRAD file.
    pub fn from_vec(mut data: Vec<u8>) -> Result<Self> {
        if is_compressed(&data) {
            data = decompress_file(&data)?;
        }

        let mut reader = Cursor::new(&data);

        let file_header: VolumeHeaderRecord = Self::decode_file_header(&mut reader)?;
        let mut file = Self::from_header(file_header);

        while reader.position() < data.len() as u64 {
            let message_header: MessageHeader = Self::deserialize(&mut reader)?;

            if message_header.msg_type() == 31 {
                Self::decode_message_31(&mut reader, &mut file)?;
            } else {
                let ff_distance = i64::try_from(2432 - size_of::<MessageHeader>())?;
                reader.seek(SeekFrom::Current(ff_distance))?;
            }
        }

        Ok(file)
    }

    /// Create a new data file for the specified header with no sweep data.
    pub(crate) fn from_header(file_header: VolumeHeaderRecord) -> Self {
        Self {
            volume_header: file_header,
            elevation_scans: BTreeMap::new(),
        }
    }

    /// The volume/file header information.
    #[must_use]
    pub fn volume_header(&self) -> &VolumeHeaderRecord {
        &self.volume_header
    }

    /// Scan data grouped by elevation number.
    #[must_use]
    pub fn elevation_scans(&self) -> &BTreeMap<u8, Vec<Message31>> {
        &self.elevation_scans
    }

    /// Scan data grouped by elevation number.
    #[must_use]
    pub fn as_elevation_scans(self) -> BTreeMap<u8, Vec<Message31>> {
        let scans = self.elevation_scans;

        // For each scan, sort the azm values
        scans
            .into_iter()
            .map(|(k, mut v)| {
                v.sort_by(|a, b| {
                    a.header()
                        .azm()
                        .partial_cmp(&b.header().azm())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                (k, v)
            })
            .collect()
    }

    /// Scan data grouped by elevation number.
    pub(crate) fn elevation_scans_mut(&mut self) -> &mut BTreeMap<u8, Vec<Message31>> {
        &mut self.elevation_scans
    }

    /// First available header for the specified elevation.
    #[must_use]
    pub fn first_volume_data(&self) -> Option<VolumeData> {
        let header = self
            .elevation_scans
            .first_key_value()?
            .1
            .first()?
            .volume_data()?
            .clone();

        Some(header)
    }

    fn decode_file_header<R: Read + Seek>(reader: &mut R) -> Result<VolumeHeaderRecord> {
        Self::deserialize(reader)
    }

    fn decode_message_31(reader: &mut Cursor<&Vec<u8>>, file: &mut DataFile) -> Result<()> {
        let start_pos = reader.position();

        let message_31_header: Message31Header = Self::deserialize(reader)?;
        let mut message = Message31::new(message_31_header);

        let pointers_space = message.header().data_block_count() as usize * size_of::<u32>();
        let mut pointers_raw = vec![0; pointers_space];
        reader.read_exact(&mut pointers_raw)?;

        let data_block_pointers = pointers_raw
            .chunks_exact(size_of::<u32>())
            .filter_map(|v| Some(<u32>::from_be_bytes(v.try_into().ok()?)))
            .collect::<Vec<_>>();

        for pointer in data_block_pointers {
            if pointer != u32::try_from(reader.position())? {
                reader.seek(SeekFrom::Start(start_pos + u64::from(pointer)))?;
            }

            let data_block: DataBlockHeader = Self::deserialize(reader)?;
            reader.seek(SeekFrom::Current(-4))?;

            let data_block_product = data_block.data_block_product()?;

            match data_block_product {
                DataBlockProduct::VolumeData => {
                    let data: VolumeData = Self::deserialize(reader)?;
                    message.set_volume_data(data);

                    // todo: I'm missing 8 bytes here
                    // reader.seek(SeekFrom::Current(8))?;
                }
                DataBlockProduct::ElevationData => {
                    let data: ElevationData = Self::deserialize(reader)?;
                    message.set_elevation_data(data);
                }
                DataBlockProduct::RadialData => {
                    let data: RadialData = Self::deserialize(reader)?;
                    message.set_radial_data(data);
                }
                DataBlockProduct::Reflectivity
                | DataBlockProduct::Velocity
                | DataBlockProduct::ClutterFilterProbability
                | DataBlockProduct::SpectrumWidth
                | DataBlockProduct::DifferentialReflectivity
                | DataBlockProduct::DifferentialPhase
                | DataBlockProduct::CorrelationCoefficient => {
                    let generic_data: GenericData = Self::deserialize(reader)?;

                    let mut moment_data = vec![0; generic_data.moment_size()];
                    reader.read_exact(&mut moment_data)?;

                    let data = DataMoment::new(data_block_product, generic_data, moment_data);
                    message.set_data_moment(data);
                }
            }
        }

        file.elevation_scans_mut()
            .entry(message.header().elev_num())
            .or_default()
            .push(message);

        Ok(())
    }

    /// Attempts to deserialize some struct from the provided binary reader.
    fn deserialize<R: Read + Seek, S: DeserializeOwned>(reader: &mut R) -> Result<S> {
        Ok(DefaultOptions::new()
            .with_fixint_encoding()
            .with_big_endian()
            .deserialize_from(reader.by_ref())?)
    }
}

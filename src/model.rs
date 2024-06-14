//!
//! Struct definitions for decoded NEXRAD Level II data structures.
//!

use std::{fmt::Debug, str::FromStr};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// NEXRAD data volume/file header.
#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct VolumeHeaderRecord {
    filename: [u8; 12],
    file_date: u32,
    file_time: u32,
    radar_id: [u8; 4],
}

impl VolumeHeaderRecord {
    /// Filename of the archive.
    #[must_use]
    pub fn filename(&self) -> &[u8; 12] {
        &self.filename
    }

    /// Modified Julian date of the file.
    #[must_use]
    pub fn file_date(&self) -> u32 {
        self.file_date
    }

    /// Milliseconds of day since midnight of the file.
    #[must_use]
    pub fn file_time(&self) -> u32 {
        self.file_time
    }

    /// ICAO radar identifier in ASCII.
    #[must_use]
    pub fn radar_id(&self) -> &[u8; 4] {
        &self.radar_id
    }
}

/// A NEXRAD volume message header indicating its type and size to be decoded.
#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageHeader {
    rpg: [u8; 12],
    msg_size: u16,
    channel: u8,
    msg_type: u8,
    id_seq: u16,
    msg_date: u16,
    msg_time: u32,
    num_segs: u16,
    seg_num: u16,
}

impl MessageHeader {
    /// 12 bytes inserted by RPG Communications Mgr. Ignored.
    #[must_use]
    pub fn rpg(&self) -> &[u8; 12] {
        &self.rpg
    }

    /// Message size for this segment, in halfwords.
    #[must_use]
    pub fn msg_size(&self) -> u16 {
        self.msg_size
    }

    /// RDA Redundant Channel
    #[must_use]
    pub fn channel(&self) -> u8 {
        self.channel
    }

    /// Message type. For example, 31.
    #[must_use]
    pub fn msg_type(&self) -> u8 {
        self.msg_type
    }

    /// Msg seq num = 0 to 7FFF, then roll over to 0.
    #[must_use]
    pub fn id_seq(&self) -> u16 {
        self.id_seq
    }

    /// Modified Julian date from 1/1/70.
    #[must_use]
    pub fn msg_date(&self) -> u16 {
        self.msg_date
    }

    /// Packet generation time in ms past midnight.
    #[must_use]
    pub fn msg_time(&self) -> u32 {
        self.msg_time
    }

    /// Number of segments for this message.
    #[must_use]
    pub fn num_segs(&self) -> u16 {
        self.num_segs
    }

    /// Number of this segment.
    #[must_use]
    pub fn seg_num(&self) -> u16 {
        self.seg_num
    }
}

/// Structured data for message type 31.
#[derive(Clone)]
pub struct Message31 {
    header: Message31Header,
    volume_data: Option<VolumeData>,
    elevation_data: Option<ElevationData>,
    radial_data: Option<RadialData>,
    reflectivity_data: Option<DataMoment>,
    velocity_data: Option<DataMoment>,
    sw_data: Option<DataMoment>,
    zdr_data: Option<DataMoment>,
    phi_data: Option<DataMoment>,
    rho_data: Option<DataMoment>,
    cfp_data: Option<DataMoment>,
}

impl Message31 {
    /// Create a new message 31 structure with just the header to start.
    pub(crate) fn new(header: Message31Header) -> Self {
        Self {
            header,
            volume_data: None,
            elevation_data: None,
            radial_data: None,
            reflectivity_data: None,
            velocity_data: None,
            sw_data: None,
            zdr_data: None,
            phi_data: None,
            rho_data: None,
            cfp_data: None,
        }
    }

    /// The message 31 header.
    #[must_use]
    pub fn header(&self) -> &Message31Header {
        &self.header
    }

    /// The volume data block.
    #[must_use]
    pub fn volume_data(&self) -> Option<&VolumeData> {
        self.volume_data.as_ref()
    }

    /// The elevation data block.
    #[must_use]
    pub fn elevation_data(&self) -> Option<&ElevationData> {
        self.elevation_data.as_ref()
    }

    /// The radial data block.
    #[must_use]
    pub fn radial_data(&self) -> Option<&RadialData> {
        self.radial_data.as_ref()
    }

    /// The reflectivity data block.
    #[must_use]
    pub fn reflectivity_data(&self) -> Option<&DataMoment> {
        self.reflectivity_data.as_ref()
    }

    /// The velocity data block.
    #[must_use]
    pub fn velocity_data(&self) -> Option<&DataMoment> {
        self.velocity_data.as_ref()
    }

    /// The spectrum width data block.
    #[must_use]
    pub fn sw_data(&self) -> Option<&DataMoment> {
        self.sw_data.as_ref()
    }

    /// The differential reflectivity data block.
    #[must_use]
    pub fn zdr_data(&self) -> Option<&DataMoment> {
        self.zdr_data.as_ref()
    }

    /// The differential phase data block.
    #[must_use]
    pub fn phi_data(&self) -> Option<&DataMoment> {
        self.phi_data.as_ref()
    }

    /// The correlation coefficient data block.
    #[must_use]
    pub fn rho_data(&self) -> Option<&DataMoment> {
        self.rho_data.as_ref()
    }

    /// The clutter filter power data block.
    #[must_use]
    pub fn cfp_data(&self) -> Option<&DataMoment> {
        self.cfp_data.as_ref()
    }

    #[must_use]
    pub fn get_data_moment(&self, product: &DataBlockProduct) -> Option<&DataMoment> {
        match product {
            DataBlockProduct::Reflectivity => self.reflectivity_data(),
            DataBlockProduct::Velocity => self.velocity_data(),
            DataBlockProduct::SpectrumWidth => self.sw_data(),
            DataBlockProduct::DifferentialReflectivity => self.zdr_data(),
            DataBlockProduct::DifferentialPhase => self.phi_data(),
            DataBlockProduct::CorrelationCoefficient => self.rho_data(),
            DataBlockProduct::ClutterFilterProbability => self.cfp_data(),
            DataBlockProduct::VolumeData
            | DataBlockProduct::ElevationData
            | DataBlockProduct::RadialData => None,
        }
    }

    /// Set data based on `DataMoment`
    pub(crate) fn set_data_moment(&mut self, data_moment: DataMoment) {
        match data_moment.product {
            DataBlockProduct::Reflectivity => self.reflectivity_data = Some(data_moment),
            DataBlockProduct::Velocity => self.velocity_data = Some(data_moment),
            DataBlockProduct::SpectrumWidth => self.sw_data = Some(data_moment),
            DataBlockProduct::DifferentialReflectivity => self.zdr_data = Some(data_moment),
            DataBlockProduct::DifferentialPhase => self.phi_data = Some(data_moment),
            DataBlockProduct::CorrelationCoefficient => self.rho_data = Some(data_moment),
            DataBlockProduct::ClutterFilterProbability => self.cfp_data = Some(data_moment),
            DataBlockProduct::VolumeData
            | DataBlockProduct::ElevationData
            | DataBlockProduct::RadialData => {}
        }
    }

    /// Set the volume data block.
    pub(crate) fn set_volume_data(&mut self, volume_data: VolumeData) {
        self.volume_data = Some(volume_data);
    }

    /// Set the elevation data block.
    pub(crate) fn set_elevation_data(&mut self, elevation_data: ElevationData) {
        self.elevation_data = Some(elevation_data);
    }

    /// Set the radial data block.
    pub(crate) fn set_radial_data(&mut self, radial_data: RadialData) {
        self.radial_data = Some(radial_data);
    }
}

/// Header for message type 31.
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message31Header {
    radar_id: [u8; 4],
    ray_time: u32,
    ray_date: u16,
    azm_num: u16,
    azm: f32,
    compression_code: u8,
    spare: u8,
    radial_len: u16,
    azm_res: u8,
    radial_status: u8,
    elev_num: u8,
    sector_cut_num: u8,
    elev: f32,
    radial_spot_blanking: u8,
    azm_indexing_mode: u8,
    data_block_count: u16,
}

impl Message31Header {
    /// Radar site identifier.
    #[must_use]
    pub fn radar_id(&self) -> &[u8; 4] {
        &self.radar_id
    }

    /// Data collection time in milliseconds past midnight GMT.
    #[must_use]
    pub fn ray_time(&self) -> u32 {
        self.ray_time
    }

    /// Julian date - 2440586.5 (1/01/1970).
    #[must_use]
    pub fn ray_date(&self) -> u16 {
        self.ray_date
    }

    /// Radial number within elevation scan.
    #[must_use]
    pub fn azm_num(&self) -> u16 {
        self.azm_num
    }

    /// Azimuth angle in degrees (0 to 359.956055).
    #[must_use]
    pub fn azm(&self) -> f32 {
        self.azm
    }

    /// 0 = uncompressed, 1 = BZIP2, 2 = zlib.
    #[must_use]
    pub fn compression_code(&self) -> u8 {
        self.compression_code
    }

    /// For word alignment.
    #[must_use]
    pub fn spare(&self) -> u8 {
        self.spare
    }

    /// Radial length in bytes, including data header block.
    #[must_use]
    pub fn radial_len(&self) -> u16 {
        self.radial_len
    }

    /// Azimuthal resolution.
    #[must_use]
    pub fn azm_res(&self) -> u8 {
        self.azm_res
    }

    /// Radial status.
    #[must_use]
    pub fn radial_status(&self) -> u8 {
        self.radial_status
    }

    /// Elevation number.
    #[must_use]
    pub fn elev_num(&self) -> u8 {
        self.elev_num
    }

    /// Sector cut number.
    #[must_use]
    pub fn sector_cut_num(&self) -> u8 {
        self.sector_cut_num
    }

    /// Elevation angle in degrees (-7.0 to 70.0).
    #[must_use]
    pub fn elev(&self) -> f32 {
        self.elev
    }

    /// Radial spot blanking.
    #[must_use]
    pub fn radial_spot_blanking(&self) -> u8 {
        self.radial_spot_blanking
    }

    /// Azimuth indexing mode.
    #[must_use]
    pub fn azm_indexing_mode(&self) -> u8 {
        self.azm_indexing_mode
    }

    /// Data block count.
    #[must_use]
    pub fn data_block_count(&self) -> u16 {
        self.data_block_count
    }
}

/// Introduces a data block containing data, such as VEL, REF, etc.
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataBlockHeader {
    data_block_type: [u8; 1],
    data_name: [u8; 3],
}

impl DataBlockHeader {
    #[must_use]
    pub fn data_block_type(&self) -> &[u8; 1] {
        &self.data_block_type
    }

    /// Data name, e.g. "REF", "VEL", etc.
    #[must_use]
    pub fn data_name(&self) -> &[u8; 3] {
        &self.data_name
    }

    /// Data block header name
    ///
    /// # Errors
    /// Will error if the data block product is not recognized.
    pub fn data_block_product(&self) -> Result<DataBlockProduct> {
        Ok(DataBlockProduct::from_str(
            String::from_utf8_lossy(self.data_name()).as_ref(),
        )?)
    }
}

#[derive(Clone)]
pub enum DataBlockProduct {
    Reflectivity,
    Velocity,
    SpectrumWidth,
    DifferentialReflectivity,
    DifferentialPhase,
    CorrelationCoefficient,
    ClutterFilterProbability,

    VolumeData,
    ElevationData,
    RadialData,
}

impl FromStr for DataBlockProduct {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "REF" => Ok(Self::Reflectivity),
            "VEL" => Ok(Self::Velocity),
            "SW " => Ok(Self::SpectrumWidth),
            "ZDR" => Ok(Self::DifferentialReflectivity),
            "PHI" => Ok(Self::DifferentialPhase),
            "RHO" => Ok(Self::CorrelationCoefficient),
            "CFP" => Ok(Self::ClutterFilterProbability),
            "VOL" => Ok(Self::VolumeData),
            "RAD" => Ok(Self::RadialData),
            "ELV" => Ok(Self::ElevationData),
            _ => Err(Error::UnhandledProduct),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Product {
    Reflectivity,
    Velocity,
    SpectrumWidth,
    DifferentialReflectivity,
    DifferentialPhase,
    CorrelationCoefficient,
    ClutterFilterProbability,
}

impl FromStr for Product {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ref" => Ok(Self::Reflectivity),
            "vel" => Ok(Self::Velocity),
            "sw " => Ok(Self::SpectrumWidth),
            "zdr" => Ok(Self::DifferentialReflectivity),
            "phi" => Ok(Self::DifferentialPhase),
            "rho" => Ok(Self::CorrelationCoefficient),
            "cfp" => Ok(Self::ClutterFilterProbability),
            _ => Err(Error::UnhandledProduct),
        }
    }
}

impl From<Product> for DataBlockProduct {
    fn from(product: Product) -> Self {
        match product {
            Product::Reflectivity => Self::Reflectivity,
            Product::Velocity => Self::Velocity,
            Product::SpectrumWidth => Self::SpectrumWidth,
            Product::DifferentialReflectivity => Self::DifferentialReflectivity,
            Product::DifferentialPhase => Self::DifferentialPhase,
            Product::CorrelationCoefficient => Self::CorrelationCoefficient,
            Product::ClutterFilterProbability => Self::ClutterFilterProbability,
        }
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VolumeData {
    data_block_header: DataBlockHeader,
    lrtup: u16,
    version_major: u8,
    version_minor: u8,
    lat: f32,
    long: f32,
    site_height: u16,
    feedhorn_height: u16,
    calibration_constant: f32,
    shvtx_power_hor: f32,
    shvtx_power_ver: f32,
    system_differential_reflectivity: f32,
    initial_system_differential_phase: f32,
    volume_coverage_pattern_number: u16,
    processing_status: u16,
}

impl VolumeData {
    #[must_use]
    pub fn data_block_header(&self) -> &DataBlockHeader {
        &self.data_block_header
    }

    #[must_use]
    pub fn lrtup(&self) -> u16 {
        self.lrtup
    }

    #[must_use]
    pub fn version_major(&self) -> u8 {
        self.version_major
    }

    #[must_use]
    pub fn version_minor(&self) -> u8 {
        self.version_minor
    }

    #[must_use]
    pub fn lat(&self) -> f32 {
        self.lat
    }

    #[must_use]
    pub fn long(&self) -> f32 {
        self.long
    }

    #[must_use]
    pub fn site_height(&self) -> u16 {
        self.site_height
    }

    #[must_use]
    pub fn feedhorn_height(&self) -> u16 {
        self.feedhorn_height
    }

    #[must_use]
    pub fn calibration_constant(&self) -> f32 {
        self.calibration_constant
    }

    #[must_use]
    pub fn shvtx_power_hor(&self) -> f32 {
        self.shvtx_power_hor
    }

    #[must_use]
    pub fn shvtx_power_ver(&self) -> f32 {
        self.shvtx_power_ver
    }

    #[must_use]
    pub fn system_differential_reflectivity(&self) -> f32 {
        self.system_differential_reflectivity
    }

    #[must_use]
    pub fn initial_system_differential_phase(&self) -> f32 {
        self.initial_system_differential_phase
    }

    #[must_use]
    pub fn volume_coverage_pattern_number(&self) -> u16 {
        self.volume_coverage_pattern_number
    }

    #[must_use]
    pub fn processing_status(&self) -> u16 {
        self.processing_status
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ElevationData {
    data_block_header: DataBlockHeader,
    lrtup: u16,
    atmos: [u8; 2],
    calib_const: f32,
}

impl ElevationData {
    #[must_use]
    pub fn data_block_header(&self) -> &DataBlockHeader {
        &self.data_block_header
    }

    /// Size of data block in bytes
    #[must_use]
    pub fn lrtup(&self) -> u16 {
        self.lrtup
    }

    /// Atmospheric Attenuation Factor
    #[must_use]
    pub fn atmos(&self) -> &[u8; 2] {
        &self.atmos
    }

    /// Scaling constant used by the Signal Processor for this elevation to calculate reflectivity
    #[must_use]
    pub fn calib_const(&self) -> f32 {
        self.calib_const
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RadialData {
    data_block_header: DataBlockHeader,
    lrtup: u16,
    unambiguous_range: u16,
    noise_level_horz: f32,
    noise_level_vert: f32,
    nyquist_velocity: u16,
    radial_flags: u16,
    calib_const_horz_chan: f32,
    calib_const_vert_chan: f32,
}

impl RadialData {
    #[must_use]
    pub fn data_block_header(&self) -> &DataBlockHeader {
        &self.data_block_header
    }

    /// Size of data block in bytes
    #[must_use]
    pub fn lrtup(&self) -> u16 {
        self.lrtup
    }

    /// Unambiguous Range, Interval Size
    #[must_use]
    pub fn unambiguous_range(&self) -> u16 {
        self.unambiguous_range
    }

    #[must_use]
    pub fn noise_level_horz(&self) -> f32 {
        self.noise_level_horz
    }

    #[must_use]
    pub fn noise_level_vert(&self) -> f32 {
        self.noise_level_vert
    }

    #[must_use]
    pub fn nyquist_velocity(&self) -> u16 {
        self.nyquist_velocity
    }

    #[must_use]
    pub fn radial_flags(&self) -> u16 {
        self.radial_flags
    }

    #[must_use]
    pub fn calib_const_horz_chan(&self) -> f32 {
        self.calib_const_horz_chan
    }

    #[must_use]
    pub fn calib_const_vert_chan(&self) -> f32 {
        self.calib_const_vert_chan
    }
}

#[derive(Clone)]
pub struct DataMoment {
    product: DataBlockProduct,
    data: GenericData,
    moment_data: Vec<u8>,
}

impl DataMoment {
    pub(crate) fn new(product: DataBlockProduct, data: GenericData, moment_data: Vec<u8>) -> Self {
        Self {
            product,
            data,
            moment_data,
        }
    }

    #[must_use]
    pub fn data(&self) -> &GenericData {
        &self.data
    }

    #[must_use]
    pub fn moment_data(&self) -> &[u8] {
        &self.moment_data
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericData {
    data_block_type: [u8; 1],
    data_name: [u8; 3],
    reserved: u32,
    number_data_moment_gates: u16,
    data_moment_range: u16,
    data_moment_range_sample_interval: u16,
    tover: u16,
    snr_threshold: u16,
    control_flags: u8,
    data_word_size: u8,
    scale: f32,
    offset: f32,
}

impl GenericData {
    #[must_use]
    pub fn data_block_type(&self) -> &[u8; 1] {
        &self.data_block_type
    }

    #[must_use]
    pub fn data_name(&self) -> &[u8; 3] {
        &self.data_name
    }

    #[must_use]
    pub fn reserved(&self) -> u32 {
        self.reserved
    }

    /// Number of data moment gates for current radial
    #[must_use]
    pub fn number_data_moment_gates(&self) -> u16 {
        self.number_data_moment_gates
    }

    /// Range to center of first range gate
    #[must_use]
    pub fn data_moment_range(&self) -> u16 {
        self.data_moment_range
    }

    /// Size of data moment sample interval
    #[must_use]
    pub fn data_moment_range_sample_interval(&self) -> u16 {
        self.data_moment_range_sample_interval
    }

    /// Threshold parameter which specifies the minimum difference in echo power between two
    /// resolution gates for them not to be labeled "overlayed"
    #[must_use]
    pub fn tover(&self) -> u16 {
        self.tover
    }

    /// SNR threshold for valid data
    #[must_use]
    pub fn snr_threshold(&self) -> u16 {
        self.snr_threshold
    }

    /// Indicates special control features
    #[must_use]
    pub fn control_flags(&self) -> u8 {
        self.control_flags
    }

    /// Number of bits (DWS) used for storing data for each Data Moment gate
    #[must_use]
    pub fn data_word_size(&self) -> u8 {
        self.data_word_size
    }

    /// Scale value used to convert Data Moments from integer to floating point data
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Offset value used to convert Data Moments from integer to floating point data
    #[must_use]
    pub fn offset(&self) -> f32 {
        self.offset
    }

    #[must_use]
    pub fn moment_size(&self) -> usize {
        self.number_data_moment_gates as usize * self.data_word_size as usize / 8
    }
}

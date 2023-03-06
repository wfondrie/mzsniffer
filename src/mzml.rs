// A lot of this is from Sage: https://github.com/lazear/sage
use async_compression::tokio::bufread::ZlibDecoder;
use quick_xml::events::Event;
use quick_xml::Reader;
use tokio::io::{AsyncBufRead, AsyncReadExt};

#[derive(Default, Debug, Clone)]
pub struct Spectrum {
    /// Scan start time
    pub scan_start_time: f32,
    /// Total ion current
    pub total_ion_current: f32,
    /// m/z array
    pub mz_array: Vec<f64>,
    /// Intensity array
    pub intensity_array: Vec<f64>,
}

// MUST supply only one of the following
const ZLIB_COMPRESSION: &str = "MS:1000574";
const NO_COMPRESSION: &str = "MS:1000576";

// MUST supply only one of the following
const INTENSITY_ARRAY: &str = "MS:1000515";
const MZ_ARRAY: &str = "MS:1000514";

// MUST supply only one of the following
const FLOAT_64: &str = "MS:1000523";
const FLOAT_32: &str = "MS:1000521";

// MS info:
const MS_LEVEL: &str = "MS:1000511";
const PROFILE: &str = "MS:1000128";
const CENTROID: &str = "MS:1000127";
const TOTAL_ION_CURRENT: &str = "MS:1000285";
const SCAN_START_TIME: &str = "MS:1000016";

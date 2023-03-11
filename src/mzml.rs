// This is almost all directly from Sage (MIT License):
// https://github.com/lazear/sage/blob/46c3210af1d49fd9b7b5935ebae202bfd905bda1/crates/sage/src/mzml.rs
use async_compression::tokio::bufread::ZlibDecoder;
use quick_xml::events::Event;
use quick_xml::Reader;
use tokio::io::{AsyncBufRead, AsyncReadExt};

#[derive(Default, Debug, Clone)]
pub struct Spectrum {
    pub ms_level: u8,
    pub id: String,
    pub representation: Representation,
    pub scan_start_time: f64,
    pub total_ion_current: f64,
    pub mz: Vec<f64>,
    pub intensity: Vec<f64>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Representation {
    Profile,
    Centroid,
}

impl Default for Representation {
    fn default() -> Self {
        Self::Profile
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Which tag are we inside?
enum State {
    Spectrum,
    Scan,
    BinaryDataArray,
    Binary,
    Precursor,
}

#[derive(Copy, Clone, Debug)]
enum BinaryKind {
    Intensity,
    Mz,
}

#[derive(Copy, Clone, Debug)]
enum Dtype {
    F32,
    F64,
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

// Spectrum cvParams:
const MS_LEVEL: &str = "MS:1000511";
const PROFILE: &str = "MS:1000128";
const CENTROID: &str = "MS:1000127";
const TOTAL_ION_CURRENT: &str = "MS:1000285";
const SCAN_START_TIME: &str = "MS:1000016";
const SCAN_WINDOW_LOWER: &str = "MS:1000501";
const SCAN_WINDOW_UPPER: &str = "MS:1000500";


#[derive(Debug)]
pub struct MS1Spectra {
   pub spectra: Vec<Spectrum>,
   pub scan_range: (f64, f64),
}

pub struct MzMLReader;

impl MzMLReader {
    /// Create a new [`MzMlReader`].
    pub fn new() -> Self { Self }

    /// Here be dragons -
    /// Seriously, this kinda sucks because it's a giant imperative, stateful loop.
    /// But I also don't want to spend any more time working on an mzML parser...
    pub async fn parse<B: AsyncBufRead + Unpin>(&self, b: B) -> Result<MS1Spectra, MzMLError> {
        let mut reader = Reader::from_reader(b);
        let mut buf = Vec::new();

        let mut state = None;
        let mut compression = false;
        let mut output_buffer = Vec::with_capacity(4096);
        let mut binary_dtype = Dtype::F64;
        let mut binary_array = None;

        let mut spectrum = Spectrum::default();
        let mut spectra = Vec::new();
        let mut scan_range = (0., 0.);

        macro_rules! extract {
            ($ev:expr, $key:expr) => {
                $ev.try_get_attribute($key)?
                    .ok_or_else(|| MzMLError::Malformed)?
                    .value
            };
        }

        loop {
            match reader.read_event_into_async(&mut buf).await {
                Ok(Event::Start(ref ev)) => {
                    // State transition into child tag
                    state = match (ev.name().into_inner(), state) {
                        (b"spectrum", _) => Some(State::Spectrum),
                        (b"scan", Some(State::Spectrum)) => Some(State::Scan),
                        (b"binaryDataArray", Some(State::Spectrum)) => Some(State::BinaryDataArray),
                        (b"binary", Some(State::BinaryDataArray)) => Some(State::Binary),
                        (b"precursor", Some(State::Spectrum)) => Some(State::Precursor),
                        _ => state,
                    };
                    match ev.name().into_inner() {
                        b"spectrum" => {
                            let id = extract!(ev, b"id");
                            let id = std::str::from_utf8(&id)?;
                            spectrum.id = id.to_string();
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref ev)) => match (state, ev.name().into_inner()) {
                    (Some(State::BinaryDataArray), b"cvParam") => {
                        let accession = extract!(ev, b"accession");
                        let accession = std::str::from_utf8(&accession)?;
                        match accession {
                            ZLIB_COMPRESSION => compression = true,
                            NO_COMPRESSION => compression = false,
                            FLOAT_64 => binary_dtype = Dtype::F64,
                            FLOAT_32 => binary_dtype = Dtype::F32,
                            INTENSITY_ARRAY => binary_array = Some(BinaryKind::Intensity),
                            MZ_ARRAY => binary_array = Some(BinaryKind::Mz),
                            _ => {
                                // Unknown CV - perhaps noise
                                binary_array = None;
                            }
                        }
                    }
                    (Some(State::Spectrum), b"cvParam") => {
                        let accession = extract!(ev, b"accession");
                        let accession = std::str::from_utf8(&accession)?;
                        match accession {
                            MS_LEVEL => {
                                let level = extract!(ev, b"value");
                                let level = std::str::from_utf8(&level)?.parse::<u8>()?;
                                if level != 1 {
                                    spectrum = Spectrum::default();
                                    state = None;
                                } else {
                                    spectrum.ms_level = level;
                                }
                            }
                            PROFILE => spectrum.representation = Representation::Profile,
                            CENTROID => spectrum.representation = Representation::Centroid,
                            TOTAL_ION_CURRENT => {
                                let value = extract!(ev, b"value");
                                let value = std::str::from_utf8(&value)?.parse::<f64>()?;
                                if value == 0.0 {
                                    // No ion current, break out of current state
                                    spectrum = Spectrum::default();
                                    state = None;
                                } else {
                                    spectrum.total_ion_current = value;
                                }
                            }
                            _ => {}
                        }
                    }
                    (Some(State::Scan), b"cvParam") => {
                        let accession = extract!(ev, b"accession");
                        let accession = std::str::from_utf8(&accession)?;
                        let value = extract!(ev, b"value");
                        let value = std::str::from_utf8(&value)?;
                        match accession {
                            SCAN_START_TIME => {
                                spectrum.scan_start_time = value.parse()?;
                            }
                            SCAN_WINDOW_LOWER => {
                                let mz = value.parse()?;
                                if mz < scan_range.0 {
                                    scan_range.0 = mz;
                                }
                            }
                            SCAN_WINDOW_UPPER => {
                                let mz = value.parse()?;
                                if mz > scan_range.1 {
                                    scan_range.1 = mz;
                                }
                            }
                            _ => {}
                        }
                    }

                    _ => {}
                },
                Ok(Event::Text(text)) => {
                    if let Some(State::Binary) = state {
                        if spectrum.ms_level != 1 {
                            continue;
                        }
                        let raw = text.unescape()?;
                        // There are occasionally empty binary data arrays, or unknown CVs
                        if raw.is_empty() || binary_array.is_none() {
                            continue;
                        }
                        let decoded = base64::decode(raw.as_bytes())?;
                        let bytes = match compression {
                            false => &decoded,
                            true => {
                                let mut r = ZlibDecoder::new(decoded.as_slice());
                                let n = r.read_to_end(&mut output_buffer).await?;
                                &output_buffer[..n]
                            }
                        };

                        let array = match binary_dtype {
                            Dtype::F32 => {
                                let mut buf: [u8; 4] = [0; 4];
                                bytes
                                    .chunks(4)
                                    .filter(|chunk| chunk.len() == 4)
                                    .map(|chunk| {
                                        buf.copy_from_slice(chunk);
                                        f32::from_le_bytes(buf) as f64
                                    })
                                    .collect::<Vec<f64>>()
                            }
                            Dtype::F64 => {
                                let mut buf: [u8; 8] = [0; 8];
                                bytes
                                    .chunks(8)
                                    .map(|chunk| {
                                        buf.copy_from_slice(chunk);
                                        f64::from_le_bytes(buf)
                                    })
                                    .collect::<Vec<f64>>()
                            }
                        };
                        output_buffer.clear();

                        match binary_array {
                            Some(BinaryKind::Intensity) => {
                                spectrum.intensity = array;
                            }
                            Some(BinaryKind::Mz) => {
                                spectrum.mz = array;
                            }
                            None => {}
                        }

                        binary_array = None;
                    }
                }
                Ok(Event::End(ev)) => {
                    state = match (state, ev.name().into_inner()) {
                        (Some(State::Binary), b"binary") => Some(State::BinaryDataArray),
                        (Some(State::BinaryDataArray), b"binaryDataArray") => Some(State::Spectrum),
                        (Some(State::Scan), b"scan") => Some(State::Spectrum),
                        (_, b"spectrum") => {
                            if spectrum.ms_level == 1 {
                                spectra.push(spectrum);
                            }
                            spectrum = Spectrum::default();
                            None
                        }
                        _ => state,
                    };
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(err) => {
                    log::error!("unhandled XML error while parsing mzML: {}", err)
                }
            }
            buf.clear();
        }

        let out = MS1Spectra{
            spectra,
            scan_range,
        };
        Ok(out)
    }
}


#[derive(Debug)]
pub enum MzMLError {
    Malformed,
    UnsupportedCV(String),
    XMLError(quick_xml::Error),
    IOError(std::io::Error),
}

impl std::fmt::Display for MzMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MzMLError::Malformed => f.write_str("MzMLError: malformed cvParam"),
            MzMLError::UnsupportedCV(s) => write!(f, "MzMLError: unsupported cvParam {}", s),
            MzMLError::IOError(s) => write!(f, "MzMLError: IO error {}", s),
            MzMLError::XMLError(s) => write!(f, "MzMLError: XML error {}", s),
        }
    }
}

impl std::error::Error for MzMLError {}

impl From<std::io::Error> for MzMLError {
    fn from(residual: std::io::Error) -> Self {
        Self::IOError(residual)
    }
}

impl From<quick_xml::Error> for MzMLError {
    fn from(residual: quick_xml::Error) -> Self {
        Self::XMLError(residual)
    }
}

impl From<std::str::Utf8Error> for MzMLError {
    fn from(_: std::str::Utf8Error) -> Self {
        Self::Malformed
    }
}

impl From<std::num::ParseFloatError> for MzMLError {
    fn from(_: std::num::ParseFloatError) -> Self {
        Self::Malformed
    }
}

impl From<std::num::ParseIntError> for MzMLError {
    fn from(_: std::num::ParseIntError) -> Self {
        Self::Malformed
    }
}

impl From<base64::DecodeError> for MzMLError {
    fn from(_: base64::DecodeError) -> Self {
        Self::Malformed
    }
}

#[cfg(test)]
mod tests {
    use super::MzMLReader;
    use tokio::fs::File;
    use tokio::io::BufReader;
    const TEST_FILE: &str = "data/MSV000081544.20170728_MS1_17k_plasmaspikedPEG_3.mzML";

    #[tokio::test]
    async fn smoke() {
        let mzml_file = File::open(TEST_FILE).await.unwrap();
        let mzml_file = BufReader::new(mzml_file);
        let res = MzMLReader::new().parse(mzml_file).await.unwrap();
        assert_eq!(res.spectra[0].mz.len(), 435);
        assert_eq!(res.scan_range, (0.0, 1500.0));
    }
}

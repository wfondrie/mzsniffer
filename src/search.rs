use std::error::Error;

use rayon::prelude::*;
use serde::Serialize;

use crate::polymer::Polymer;
use crate::mzml::MS1Spectra;
use crate::defaults::DefaultPolymers;

#[derive(Serialize, Clone, Debug)]
pub struct PolymerResults {
    pub filename: String,
    pub polymers: Vec<PolymerResult>,
    pub ret_times: Vec<f64>,
    pub tic: Vec<f64>,
    pub total: f64,
}


#[derive(Serialize, Clone, Debug)]
pub struct PolymerResult {
    pub name: String,
    pub total: f64,
    pub xic: Vec<f64>,
}

impl PolymerResult {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            total: 0.,
            xic: Vec::new(),
        }
    }
}


pub fn search(
    filename: String,
    spec: MS1Spectra,
    tol: &f64,
    unit: &str
) -> Result<PolymerResults, SearchError> {
    let polymers = DefaultPolymers::new();
    let poly_results: Vec<PolymerResult> = polymers.0
        .into_par_iter()
        .map(|mut x| search_for_polymer(&mut x, &spec, tol, unit))
        .collect::<Result<Vec<PolymerResult>, SearchError>>()?;

    let mut results = PolymerResults {
        filename,
        polymers: poly_results,
        ret_times: Vec::new(),
        tic: Vec::new(),
        total: 0.,
    };

    let _ = spec.spectra
        .into_iter()
        .map(|x| {
            results.ret_times.push(x.scan_start_time);
            results.tic.push(x.total_ion_current);
            results.total += x.total_ion_current;
        }).count();

    Ok(results)
}


fn search_for_polymer(
    poly: &mut Polymer,
    spec: &MS1Spectra,
    tol: &f64,
    unit: &str,
) -> Result<PolymerResult, SearchError> {
    poly.calculate_bounds(&spec.scan_range.1, tol, unit);
    let mut results = PolymerResult::new(&poly.name);

    results.xic = spec.spectra
        .to_vec()
        .into_par_iter()
        .map(
            |x| find_peaks(
                &poly.precursors.as_ref().unwrap(),
                &poly.tols.as_ref().unwrap(),
                &x.mz,
                &x.intensity,
            )
        )
        .collect::<Vec<f64>>();

    results.total = results.xic.clone().into_iter().sum();
    Ok(results)
}

fn find_peaks(
    query_vec: &Vec<f64>,
    tol_vec: &Vec<f64>,
    mz_vec: &Vec<f64>,
    intensity_vec: &Vec<f64>,
) -> f64 {
    let mut total_intensity = 0.;
    let query_iter = query_vec
        .into_iter()
        .zip(tol_vec.into_iter());

    for (query_mz, tol) in query_iter {
        let mut biggest = 0.;
        let spec_iter = mz_vec
            .into_iter()
            .zip(intensity_vec.into_iter());

        for (mz, intensity) in spec_iter {
            if (mz - query_mz).abs() <= *tol && intensity > &biggest {
                biggest = *intensity;
            }
        }
        total_intensity += biggest;
    }
    total_intensity
}



#[derive(Debug)]
pub struct SearchError {
    details: String
}

impl SearchError {
    fn new(msg: &str) -> SearchError {
        SearchError{details: msg.to_string()}
    }
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for SearchError {
    fn description(&self) -> &str {
        &self.details
    }
}


#[cfg(test)]
mod tests {
    use super::search;
    use crate::mzml::MzMLReader;
    use tokio::fs::File;
    use tokio::io::BufReader;
    const TEST_FILE: &str = "data/MSV000081544.20170728_MS1_17k_plasmaspikedPEG_3.mzML";

    #[tokio::test]
    async fn smoke() {
        let mzml_file = File::open(TEST_FILE).await.unwrap();
        let mzml_file = BufReader::new(mzml_file);
        let spectra = MzMLReader::new().parse(mzml_file).await.unwrap();
        search(TEST_FILE.to_string(), spectra, &10., "ppm").unwrap();
    }
}

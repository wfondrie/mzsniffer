use lazy_static::lazy_static;
use regex::Regex;

// Atoms
fn atomic_mass(atom: &str) -> f64 {
    match atom {
        "C" => 12.0000000000000,
        "H" => 1.007825032071,
        "O" => 15.9949146195616,
        "N" => 14.00307400486,
        "Na" => 22.989769282019,
        "Si" => 27.97692653505,
        _ => unreachable!("BUG: Unknown atom {}.", atom),
    }
}

// Useful constants
pub const PROTON: f64 = 1.00727646681290;
pub const NEUTRON: f64 = 1.0086649158849;

// Compute the mass from an empirical formula.
pub fn formula_mass(formula: &str) -> f64 {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"([A-Z][a-z]?)(\d*)").unwrap();
    }

    let mut total_mass = 0.;
    for cap in RE.captures_iter(formula) {
        let num = &cap[2].parse::<f64>().unwrap_or(1.);
        total_mass += num * atomic_mass(&cap[1]);
    }

    total_mass
}


pub fn mass_to_mz(mass: f64, charge: u32, protonate: bool) -> f64 {
    let mut mz = mass / charge as f64;
    if protonate {
        mz += PROTON;
    }
    mz
}


#[cfg(test)]
mod tests {
    use super::{formula_mass, mass_to_mz};

    #[test]
    fn test_formulas() {
        assert_eq!(formula_mass("H2O"), 18.0105646837036);
        assert_eq!(formula_mass("Si2H"), 56.961678102171);
    }

    #[test]
    fn test_mass_to_mz() {
        let mass = formula_mass("H2O");
        assert_eq!(mass_to_mz(mass, 1, true), 19.0178411505165);
        assert_eq!(mass_to_mz(mass, 2, true), 10.0125588086647);
    }
}

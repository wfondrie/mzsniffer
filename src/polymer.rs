use crate::mass::{formula_mass, mass_to_mz};

#[derive(Debug)]
pub struct Polymer<'a> {
    /// The name of the polymer.
    name: &'a str,
    /// The empirical formula for the non-repeating parts of the
    /// molecule.
    core_formula: &'a str,
    /// The empirical formula for the repeating part of the molecule.
    rep_formula: &'a str,
    /// The charge of the molecule.
    charge: u32,
    /// Add protons to account for the charge? If false, you should
    /// add the charged-atoms to your 'core_formula'.
    protonate: bool,
    /// The precursor m/z values.
    precursors: Option<Vec<f64>>,
    /// The tolerance around each m/z value.
    tols: Option<Vec<f64>>,
}

impl<'a> Polymer<'a> {
    pub fn new(
        name: &'a str,
        core_formula: &'a str,
        rep_formula: &'a str,
        charge: u32,
        protonate: bool,
    ) -> Self {
        Self {
            name,
            core_formula,
            rep_formula,
            charge,
            precursors: None,
            tols: None,
            protonate,
        }
    }

    fn mz_array(&self, max_mz: &f64) -> Vec<f64> {
        let core_mass = formula_mass(self.core_formula);
        let rep_mass = formula_mass(self.rep_formula);
        let mut mz_vec: Vec<f64> = Vec::new();
        for rep in 0.. {
            let poly_mass = mass_to_mz(
                core_mass + rep as f64 * rep_mass,
                self.charge,
                self.protonate,
            );

            if poly_mass > *max_mz { break }

            mz_vec.push(poly_mass);

            // In the case that it isn't actually a polymer:
            if rep_mass == 0. { break }
        }
        mz_vec
    }

    fn calculate_bounds(&mut self, max_mz: &f64, tol: &f64, unit: &str) {
        let mz_array = self.mz_array(max_mz);
        let n_vals = mz_array.len();
        let tol_vals: Vec<f64> = match &unit.to_lowercase()[..] {
            "da" => vec![*tol; n_vals],
            "ppm" => mz_array
                .clone()
                .into_iter()
                .map(|x| *tol * x / 1_000_000.0)
                .collect(),
            _ => unreachable!("Invalid unit {}", unit),
        };

        self.precursors = Some(mz_array);
        self.tols = Some(tol_vals);
    }
}


#[cfg(test)]
mod tests {
    use super::Polymer;

    #[test]
    fn test_formulas_smoke() {
        let mut poly = Polymer::new("test", "CH3", "OH", 3, true);
        poly.calculate_bounds(&100., &10., "ppm");
    }
}

use crate::polymer::Polymer;

// These are the default contaminants
#[derive(Debug)]
pub struct DefaultPolymers<'a>(Vec<Polymer<'a>>);

impl DefaultPolymers<'_> {
    pub fn new() -> Self {
        let polys = vec![
            Polymer::new("PEG+1H", "H2O", "C2H4O", 1, true),
            Polymer::new("PEG+2H", "H2O", "C2H4O", 2, true),
            Polymer::new("PEG+3H", "H2O", "C2H4O", 3, true),
            Polymer::new("PPG", "H2O", "C3H6O", 1, true),
            Polymer::new("Triton X-100", "C14H22O", "C2H4O", 1, true),
            Polymer::new("Triton X-100 (Reduced)", "C14H28O", "C2H4O", 1, true),
            Polymer::new("Triton X-100 (Na)", "C14H22ONa", "C2H4O", 1, false),
            Polymer::new("Triton X-100 (Reduced, Na)", "C14H28ONa", "C2H4O", 1, false),
            Polymer::new("Triton X-101", "C15H24O", "C2H4O", 1, true),
            Polymer::new("Triton X-101 (Reduced)", "C15H30O", "C2H4O", 1, true),
            Polymer::new("Polysiloxane", "", "C2H6SiO", 1, true),
            Polymer::new("Tween-20", "C18H34O6Na", "C2H4O", 1, false),
            Polymer::new("Tween-40", "C22H42O6Na", "C2H4O", 1, false),
            Polymer::new("Tween-60", "C24H46O6Na", "C2H4O", 1, false),
            Polymer::new("Tween-80", "C24H44O6Na", "C2H4O", 1, false),
            Polymer::new("IGEPAL", "C15H24O", "C2H4O", 1, true),
        ];
        Self(polys)
    }
}



#[cfg(test)]
mod tests {
    use super::DefaultPolymers;

    #[test]
    fn test_smoke() {
        let _ = DefaultPolymers::new();
    }
}

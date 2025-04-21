#[derive(Clone, Copy, Debug)]
pub enum Phytohormone {
    /// The amount of auxins, usually between 1 and 100 µg / kg plant material.
    ///
    /// Auxins are elongation hormones, mostly promoting growth
    Auxin(f32),
    /// The amount of cytokinins
    Cytokinin(f32),
}

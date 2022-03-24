/// Controls the quality of generated glyphs.
///
/// Generally each setting can be tweaked to generate better
/// looking glyphs at the cost of a certain performance impact.
#[derive(Debug, Clone, Copy)]
pub struct QualitySettings {
    /// The number of linear interpolation steps performed
    /// on a _quadratic bezier curve_.
    ///
    /// If the specified font does not use _quadratic splines_
    /// this setting will have no effect.
    ///
    /// Higher values result in higher polygon count.
    pub quad_interpolation_steps: u32,

    /// The number of quadratic interpolation steps performed
    /// on a _cubic bezier curve_.
    ///
    /// If the specified font does not use _cubic splines_
    /// this setting will have no effect.
    ///
    /// Higher values result in higher polygon count.
    pub cubic_interpolation_steps: u32,
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            quad_interpolation_steps: 5,
            cubic_interpolation_steps: 3,
        }
    }
}

//! Units and rack conventions shared by every crate.

/// One millimetre expressed in inches.
pub const MM_PER_INCH: f64 = 25.4;

/// Height of one rack unit, in millimetres (EIA-310 / IEC 60297).
pub const RACK_U_MM: f64 = 44.45;

/// Width of a 19-inch rack opening, in millimetres.
pub const RACK_WIDTH_19_IN_MM: f64 = 482.6;

/// Convert a physical length in millimetres to screen pixels.
#[must_use]
pub fn mm_to_px(mm: f64, scale_px_per_mm: f64) -> f32 {
    (mm * scale_px_per_mm) as f32
}

/// Convert screen pixels to a physical length in millimetres.
#[must_use]
pub fn px_to_mm(px: f32, scale_px_per_mm: f64) -> f64 {
    if scale_px_per_mm <= f64::EPSILON {
        0.0
    } else {
        f64::from(px) / scale_px_per_mm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rack_unit_matches_industry_value() {
        assert!((RACK_U_MM - 44.45).abs() < f64::EPSILON);
        assert!((RACK_U_MM * 25.0 - 1111.25).abs() < 1e-9);
    }

    #[test]
    fn pixel_conversion_round_trips() {
        let scale = 2.5;
        let px = mm_to_px(100.0, scale);
        assert!((px - 250.0).abs() < 0.001);
        assert!((px_to_mm(px, scale) - 100.0).abs() < 1e-9);
        assert_eq!(px_to_mm(10.0, 0.0), 0.0);
    }
}

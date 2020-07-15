use super::image::RGBAPLU;
use rgb::*;
use rgb::alt::*;

/// See `GammaPixel` & `ToRGBAPLU`
pub trait GammaComponent {
    fn max_value() -> usize;
    fn to_linear(&self, lut: &[f64]) -> f64;
}

/// Downsampling should be done in linear RGB color space.
///
/// Used by `ToRGBAPLU`
///
/// This trait provides gamma to linear conversion via lookup table,
/// and there's implementation for sRGB for common RGB types.
pub trait GammaPixel {
    type Component: GammaComponent;
    type Output;

    fn to_linear(&self, gamma_lut: &[f64]) -> Self::Output;

    fn make_lut() -> Vec<f64> {
        (0..Self::Component::max_value() + 1)
            .map(|i| to_linear(i as f64 / Self::Component::max_value() as f64))
            .collect()
    }
}

fn to_linear(s: f64) -> f64 {
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// RGBA Premultiplied Linear-light Unit scale
///
/// Convenience function `.to_rgbaplu()` to convert RGBA bitmaps to a format useful for DSSIM.
pub trait ToRGBAPLU {
    fn to_rgbaplu(&self) -> Vec<RGBAPLU>;
}

/// Grayscale Linear-light Unit scale
pub trait ToGLU {
    fn to_glu(&self) -> Vec<f64>;
}

impl GammaComponent for u8 {
    fn max_value() -> usize { 255 }
    fn to_linear(&self, lut: &[f64]) -> f64 {
        lut[*self as usize]
    }
}

impl GammaComponent for u16 {
    fn max_value() -> usize { 65535 }
    fn to_linear(&self, lut: &[f64]) -> f64 {
        lut[*self as usize]
    }
}

impl<M> ToGLU for [M] where M: GammaPixel<Output=f64> {
    fn to_glu(&self) -> Vec<f64> {
        let gamma_lut = M::make_lut();
        self.iter().map(|px| px.to_linear(&gamma_lut)).collect()
    }
}

impl<M> GammaPixel for RGBA<M> where M: Clone + Into<f64> + GammaComponent {
    type Component = M;
    type Output = RGBAPLU;
    fn to_linear(&self, gamma_lut: &[f64]) -> RGBAPLU {
        let a_unit = self.a.clone().into() / M::max_value() as f64;
        RGBAPLU {
            r: self.r.to_linear(gamma_lut) * a_unit,
            g: self.g.to_linear(gamma_lut) * a_unit,
            b: self.b.to_linear(gamma_lut) * a_unit,
            a: a_unit,
        }
    }
}

impl<M> GammaPixel for BGRA<M> where M: Clone + Into<f64> + GammaComponent {
    type Component = M;
    type Output = RGBAPLU;
    fn to_linear(&self, gamma_lut: &[f64]) -> RGBAPLU {
        let a_unit = self.a.clone().into() / M::max_value() as f64;
        RGBAPLU {
            r: self.r.to_linear(gamma_lut) * a_unit,
            g: self.g.to_linear(gamma_lut) * a_unit,
            b: self.b.to_linear(gamma_lut) * a_unit,
            a: a_unit,
        }
    }
}

impl<M> GammaPixel for RGB<M> where M: GammaComponent {
    type Component = M;
    type Output = RGBAPLU;
    fn to_linear(&self, gamma_lut: &[f64]) -> RGBAPLU {
        RGBAPLU {
            r: self.r.to_linear(gamma_lut),
            g: self.g.to_linear(gamma_lut),
            b: self.b.to_linear(gamma_lut),
            a: 1.0,
        }
    }
}

impl<M> GammaPixel for BGR<M> where M: GammaComponent {
    type Component = M;
    type Output = RGBAPLU;
    fn to_linear(&self, gamma_lut: &[f64]) -> RGBAPLU {
        RGBAPLU {
            r: self.r.to_linear(gamma_lut),
            g: self.g.to_linear(gamma_lut),
            b: self.b.to_linear(gamma_lut),
            a: 1.0,
        }
    }
}

impl<M> GammaPixel for lodepng::GreyAlpha<M> where M: Copy + Clone + Into<f64> + GammaComponent {
    type Component = M;
    type Output = RGBAPLU;
    fn to_linear(&self, gamma_lut: &[f64]) -> RGBAPLU {
        let a_unit = self.1.clone().into() / M::max_value() as f64;
        let g = self.0.to_linear(gamma_lut);
        RGBAPLU {
            r: g * a_unit,
            g: g * a_unit,
            b: g * a_unit,
            a: a_unit,
        }
    }
}

impl<M> GammaPixel for M where M: GammaComponent {
    type Component = M;
    type Output = f64;
    fn to_linear(&self, gamma_lut: &[f64]) -> f64 {
        self.to_linear(gamma_lut)
    }
}

impl<M> GammaPixel for lodepng::Grey<M> where M: Copy + GammaComponent {
    type Component = M;
    type Output = RGBAPLU;
    fn to_linear(&self, gamma_lut: &[f64]) -> RGBAPLU {
        let g = self.0.to_linear(gamma_lut);
        RGBAPLU {
            r: g,
            g: g,
            b: g,
            a: 1.0,
        }
    }
}

impl<P> ToRGBAPLU for [P] where P: GammaPixel<Output=RGBAPLU> {
    fn to_rgbaplu(&self) -> Vec<RGBAPLU> {
        let gamma_lut = P::make_lut();
        self.iter().map(|px| px.to_linear(&gamma_lut)).collect()
    }
}

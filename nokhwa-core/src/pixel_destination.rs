use std::fmt::{Display, Formatter};
use image::Pixel;
use crate::image::NonFloatScalarWidth;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum PixelDestination {
    Rgb8,
    Rgba8,
    Rgb16,
    Rgba16,
    Bgr8,
    Bgra8,
    Bgr16,
    Bgra16,
    Luma8,
    LumaA8,
    Luma16,
    LumaA16,
}

impl PixelDestination {
    #[must_use]
    pub fn get_by_pixel<P>() -> Option<Self>
    where
        P: Pixel,
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        match P::COLOR_MODEL {
            "RGB" => match P::Subpixel::WIDTH_BYTES {
                1 => Some(PixelDestination::Rgb8),
                2 => Some(PixelDestination::Rgb16),
                _ => None,
            },
            "RGBA" => match P::Subpixel::WIDTH_BYTES {
                1 => Some(PixelDestination::Rgba8),
                2 => Some(PixelDestination::Rgba16),
                _ => None,
            },
            "BGR" => match P::Subpixel::WIDTH_BYTES {
                1 => Some(PixelDestination::Bgr8),
                2 => Some(PixelDestination::Bgr16),
                _ => None,
            },
            "BGRA" => match P::Subpixel::WIDTH_BYTES {
                1 => Some(PixelDestination::Bgra8),
                2 => Some(PixelDestination::Bgra16),
                _ => None,
            },
            "Y" => match P::Subpixel::WIDTH_BYTES {
                1 => Some(PixelDestination::Luma8),
                2 => Some(PixelDestination::Luma16),
                _ => None,
            },
            "YA" => match P::Subpixel::WIDTH_BYTES {
                1 => Some(PixelDestination::LumaA8),
                2 => Some(PixelDestination::LumaA16),
                _ => None,
            }
            _ => None,
        }
    }
}

impl Display for PixelDestination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

use image::{ImageBuffer, Pixel, Primitive};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use bytemuck::Pod;
use num_traits::{NumCast, PrimInt};

#[derive(Debug)]
pub struct DecodedImage<Px, Meta>
where
    Px: Pixel,
    <Px as Pixel>::Subpixel: NonFloatScalarWidth,
    Meta: Debug,
{
    pub buffer: ImageBuffer<Px, Vec<Px::Subpixel>>,
    pub metadata: Meta,
}

impl<Px, Meta> DecodedImage<Px, Meta> where
    Px: Pixel,
    <Px as Pixel>::Subpixel: NonFloatScalarWidth,
    Meta: Debug {
    pub fn new(buffer: ImageBuffer<Px, Vec<Px::Subpixel>>,
               metadata: Meta) -> Self {
        Self {
            buffer,
            metadata,
        }
    }
}

impl<Px, Meta> Deref for DecodedImage<Px, Meta>
where
    Px: Pixel,
    <Px as Pixel>::Subpixel: NonFloatScalarWidth,
    Meta: Debug
{
    type Target = ImageBuffer<Px, Vec<Px::Subpixel>>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<Px, Meta> DerefMut for DecodedImage<Px, Meta>
where
    Px: Pixel,
    <Px as Pixel>::Subpixel: NonFloatScalarWidth,Meta: Debug
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

// not for safe work ;p
// TODO: add more custom integer sizes, or break our dependence on image entirely and 
// create our own imagebuffer
pub trait NonFloatScalarWidth: Debug + Primitive + PrimInt + NumCast + Pod {
    const WIDTH: u32;
}

macro_rules! impl_nfsw {
    ( $( [ ( $( $things:ty ),+ ) : $size:literal ] ),* $(,)? ) => {
        $(
        $(
        impl NonFloatScalarWidth for $things {
            const WIDTH: u32 = $size;
        }
        )+
        )*
    }
}

impl_nfsw! {
    [ (u8, i8) : 8 ],
    [ (u16, i16) : 16 ],
    [ (u32, i32) : 32 ],
    [ (u64, i64) : 64 ],
}

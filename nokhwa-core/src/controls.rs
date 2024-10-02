use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    collections::{HashMap, HashSet}
};
use std::cmp::Ordering;
use crate::ranges::{ArrayRange, Options, Range, Simple};
use crate::utils::{FailedMathOp, FallibleDiv, FallibleSub};

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ControlValidationFailure;

// TODO: Replace Controls API with Properties. (this one)
/// Properties of a Camera.
/// 
/// If the property is not supported, it is `None`.
/// Custom or platform-specific properties go into `other`
pub struct CameraProperties {
    brightness: Option<CameraPropertyDescriptor>,
    contrast: Option<CameraPropertyDescriptor>,
    hue: Option<CameraPropertyDescriptor>,
    saturation: Option<CameraPropertyDescriptor>,
    sharpness: Option<CameraPropertyDescriptor>,
    gamma: Option<CameraPropertyDescriptor>,
    white_balance: Option<CameraPropertyDescriptor>,
    backlight_compensation: Option<CameraPropertyDescriptor>,
    gain: Option<CameraPropertyDescriptor>,
    pan: Option<CameraPropertyDescriptor>,
    tilt: Option<CameraPropertyDescriptor>,
    zoom: Option<CameraPropertyDescriptor>,
    exposure: Option<CameraPropertyDescriptor>,
    iris: Option<CameraPropertyDescriptor>,
    focus: Option<CameraPropertyDescriptor>,
    facing: Option<CameraPropertyDescriptor>,
    other: HashMap<String, CameraPropertyDescriptor>,
}

impl CameraProperties {
    pub fn brightness(&self) -> Option<&CameraPropertyDescriptor> {
        self.brightness.as_ref()
    }

    pub fn contrast(&self) -> Option<&CameraPropertyDescriptor> {
        self.contrast.as_ref()
    }

    pub fn hue(&self) -> Option<&CameraPropertyDescriptor> {
        self.hue.as_ref()
    }

    pub fn saturation(&self) -> Option<&CameraPropertyDescriptor> {
        self.saturation.as_ref()
    }

    pub fn sharpness(&self) -> Option<&CameraPropertyDescriptor> {
        self.sharpness.as_ref()
    }

    pub fn gamma(&self) -> Option<&CameraPropertyDescriptor> {
        self.gamma.as_ref()
    }

    pub fn white_balance(&self) -> Option<&CameraPropertyDescriptor> {
        self.white_balance.as_ref()
    }

    pub fn backlight_compensation(&self) -> Option<&CameraPropertyDescriptor> {
        self.backlight_compensation.as_ref()
    }

    pub fn gain(&self) -> Option<&CameraPropertyDescriptor> {
        self.gain.as_ref()
    }

    pub fn pan(&self) -> Option<&CameraPropertyDescriptor> {
        self.pan.as_ref()
    }

    pub fn tilt(&self) -> Option<&CameraPropertyDescriptor> {
        self.tilt.as_ref()
    }

    pub fn zoom(&self) -> Option<&CameraPropertyDescriptor> {
        self.zoom.as_ref()
    }

    pub fn exposure(&self) -> Option<&CameraPropertyDescriptor> {
        self.exposure.as_ref()
    }

    pub fn iris(&self) -> Option<&CameraPropertyDescriptor> {
        self.iris.as_ref()
    }

    pub fn focus(&self) -> Option<&CameraPropertyDescriptor> {
        self.focus.as_ref()
    }

    pub fn facing(&self) -> Option<&CameraPropertyDescriptor> {
        self.facing.as_ref()
    }

    pub fn other(&self) -> &HashMap<String, CameraPropertyDescriptor> {
        &self.other
    }
}

/// Describes an individual property.
pub struct CameraPropertyDescriptor {
    flags: HashSet<CameraPropertyFlag>,
    platform_specific_id: Option<CameraCustomPropertyPlatformId>,
    range: CameraPropertyRange,
    value: CameraPropertyValue,
}



/// Platform Specific Camera Property. This is not useful, unless you are manually dealing with
/// camera properties in `other`.
#[derive(Clone, Debug, Hash, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum CameraCustomPropertyPlatformId {
    String(String),
    LongInteger(i128),
}

/// The flags that a camera property may have.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum CameraPropertyFlag {
    /// This is automatically set - you need not interfere
    Automatic,
    /// This is manually set - you need to interfere
    Manual,
    /// The value is set continuously by the driver.
    Continuous,
    /// The value may only be read from - any attempts to change the value will error.
    ReadOnly,
    /// The value can only be written to.
    WriteOnly,
    /// May just randomly poof out of existance.
    // FIXME: where the fuck did i find this? replace above doc with actual info.
    Volatile,
    /// While the platform/driver supports this feature,
    /// your camera does not. Setting will be ignored.
    Disabled,
}

impl Display for CameraPropertyFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Ranges (Available Options of a Camera
#[non_exhaustive]
pub enum CameraPropertyRange {
    Null,
    Boolean(Simple<bool>),
    Integer(Range<i64>),
    LongInteger(Range<i128>),
    Float(Range<f32>),
    Double(Range<f64>),
    String(Simple<String>),
    Array(ArrayRange<Box<CameraPropertyValue>>),
    Enumeration(Options<Box<CameraPropertyValue>>),
    Binary(Simple<Vec<u8>>),
    Pair(),
    Triple(rgb::RGB<f64>),
    Quadruple(Box<CameraPropertyValue>, Box<CameraPropertyValue>),
    KeyValuePair(Box<CameraPropertyValue>, Box<CameraPropertyValue>)
}

/// A possible value of
///
/// IMPORTANT: Make sure to call [`check_self()`] BEFORE any other operations!
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CameraPropertyValue {
    Null,
    Boolean(bool),
    Integer(i64),
    LongInteger(i128),
    Float(f32),
    Double(f64),
    String(String),
    Array(Vec<CameraPropertyValue>),
    EnumValue(Box<CameraPropertyValue>),
    Binary(Vec<u8>),
    Pair(Box<CameraPropertyValue>, Box<CameraPropertyValue>),
    Triple(Box<CameraPropertyValue>, Box<CameraPropertyValue>, Box<CameraPropertyValue>),
    Quadruple(Box<CameraPropertyValue>, Box<CameraPropertyValue>, Box<CameraPropertyValue>, Box<CameraPropertyValue>),
}

impl CameraPropertyValue {
    pub fn check_self(&self) -> Result<(), ControlValidationFailure> {
        if self.is_simple_type() {
            return Ok(());
        }

        match self {
            CameraPropertyValue::Array(v) => {
                for value in v {
                    if !value.is_simple_type() {
                        return Err(ControlValidationFailure);
                    }
                }
            }
            CameraPropertyValue::EnumValue(e) => {
                if !e.is_simple_type() {
                    return Err(ControlValidationFailure);
                }
            }
            CameraPropertyValue::Pair(k, v) => {
                if !k.is_simple_type() || !v.is_simple_type() {
                    return Err(ControlValidationFailure);
                }
            }
            CameraPropertyValue::Triple(x, y, z) => {
                if !x.is_simple_type() || !y.is_simple_type() || !z.is_simple_type() {
                    return Err(ControlValidationFailure);
                }
            }
            CameraPropertyValue::Quadruple(x, y, z, w) => {
                if !x.is_simple_type() || !y.is_simple_type() || !z.is_simple_type() || !w.is_simple_type() {
                    return Err(ControlValidationFailure);
                }
            }
            _ => return Err(ControlValidationFailure),
        }
        Ok(())
    }

    pub fn is_simple_type(&self) -> bool {
        if let (CameraPropertyValue::Null | CameraPropertyValue::Boolean(_) | CameraPropertyValue::Integer(_) | CameraPropertyValue::LongInteger(_) | CameraPropertyValue::Float(_)  | CameraPropertyValue::Double(_) | CameraPropertyValue::String(_) | CameraPropertyValue::Binary(_)) = self {
            return true;
        }
        false
    }
}

impl PartialEq for CameraPropertyValue {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            CameraPropertyValue::Null => {
                if let CameraPropertyValue::Null = other {
                    return true;
                }
            }
            CameraPropertyValue::Boolean(b) => {
                if let CameraPropertyValue::Boolean(ob) = other {
                    return b == ob;
                }
            }
            CameraPropertyValue::Integer(i) => {
                if let CameraPropertyValue::Integer(oi) = other {
                    return i == oi;
                }
            }
            CameraPropertyValue::LongInteger(i) => {
                if let CameraPropertyValue::LongInteger(oi) = other {
                    return i == oi;
                }
            }
            CameraPropertyValue::Float(f) => {
                if let CameraPropertyValue::Float(of) = other {
                    return f == of;
                }
            }
            CameraPropertyValue::Double(d) => {
                if let CameraPropertyValue::Double(od) = other {
                    return d == od;
                }
            }
            CameraPropertyValue::String(s) => {
                if let CameraPropertyValue::String(os) = other {
                    return s == os;
                }
            }
            CameraPropertyValue::Array(a) => {
                if let CameraPropertyValue::Array(oa) = other {
                    return a == oa;
                }
            }
            CameraPropertyValue::EnumValue(ev) => {
                if let CameraPropertyValue::EnumValue(oev) = other {
                    return ev == oev;
                }
            }
            CameraPropertyValue::Binary(bin) => {
                if let CameraPropertyValue::Binary(obin) = other {
                    return bin == obin;
                }
            }
            CameraPropertyValue::Pair(a, b) => {
                if let CameraPropertyValue::Pair(oa, ob) = other {
                    return (a == oa) && (b == ob)
                }
            }
            CameraPropertyValue::Triple(x, y, z) => {
                if let CameraPropertyValue::Triple(ox, oy, oz) = other {
                    return (x == ox) && (y == oy) && (z == oz)
                }
            }
            CameraPropertyValue::Quadruple(x, y, z, w) => {
                if let CameraPropertyValue::Quadruple(ox, oy, oz, ow) = other {
                    return (x == ox) && (y == oy) && (z == oz) && (w == ow)
                }
            }
        }
        false
    }
}

impl PartialOrd for CameraPropertyValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            CameraPropertyValue::Null => {
                match other {
                    CameraPropertyValue::Null => Some(Ordering::Greater),
                    _ => Some(Ordering::Less)
                }
            }
            CameraPropertyValue::Boolean(b) => {
                match other {
                    CameraPropertyValue::Null => Some(Ordering::Greater),
                    CameraPropertyValue::Boolean(o) => {
                        if o == b {
                            Some(Ordering::Equal)
                        } else if o {
                            Some(Ordering::Less)
                        } else {
                            Some(Ordering::Greater)
                        }
                    }
                    _ => Some(Ordering::Less)
                }
            }
            CameraPropertyValue::Integer(int) => {
                match other {
                    CameraPropertyValue::Null | CameraPropertyValue::Boolean(_) => Some(Ordering::Greater),
                    CameraPropertyValue::Integer(oth) => {
                        Some(int.cmp(oth))
                    }
                    CameraPropertyValue::LongInteger(li) => {
                        let long = match i64::try_from(li) {
                            Ok(v) => v,
                            Err(_) => return None,
                        };
                        Some(int.cmp(&long))
                    }
                    _ => Some(Ordering::Less),
                }
            }
            CameraPropertyValue::LongInteger(long) => {
                match other {
                    CameraPropertyValue::Null | CameraPropertyValue::Boolean(_) => Some(Ordering::Greater),
                    CameraPropertyValue::Integer(oth) => {
                        Some(long.cmp(&(i128::from(oth))))
                    }
                    CameraPropertyValue::LongInteger(o) => {
                        Some(long.cmp(o))
                    }
                    _ => Some(Ordering::Less),
                }
            }
            CameraPropertyValue::Float(fl) => {
                match other {
                    CameraPropertyValue::Null |
                    CameraPropertyValue::Boolean(_) |
                    CameraPropertyValue::Integer(_) |
                    CameraPropertyValue::LongInteger(_) => Some(Ordering::Greater),
                    CameraPropertyValue::Float(f) => {
                        fl.partial_cmp(f)
                    }
                    CameraPropertyValue::Double(d) => {
                        f64::from(fl).partial_cmp(d)
                    }
                    _ => Some(Ordering::Less),
                }
            }
            CameraPropertyValue::Double(d) => {
                match other {
                    CameraPropertyValue::Null |
                    CameraPropertyValue::Boolean(_) |
                    CameraPropertyValue::Integer(_) |
                    CameraPropertyValue::LongInteger(_) => Some(Ordering::Greater),
                    CameraPropertyValue::Float(f) => {
                        d.partial_cmp(&(f64::from(f)))
                    }
                    CameraPropertyValue::Double(o) => {
                        d.partial_cmp(o)
                    }
                    _ => Some(Ordering::Less),
                }
            }
            CameraPropertyValue::String(s) => {
                match other {
                    CameraPropertyValue::Null |
                    CameraPropertyValue::Boolean(_) |
                    CameraPropertyValue::Integer(_) |
                    CameraPropertyValue::LongInteger(_) |
                    CameraPropertyValue::Float(_) |
                    CameraPropertyValue::Double(_) => Some(Ordering::Greater),
                    CameraPropertyValue::String(os) => {
                        s.partial_cmp(os)
                    }
                    _ => Some(Ordering::Less),
                }
            }
            CameraPropertyValue::Array(a) => {
                match other {
                    CameraPropertyValue::Null |
                    CameraPropertyValue::Boolean(_) |
                    CameraPropertyValue::Integer(_) |
                    CameraPropertyValue::LongInteger(_) |
                    CameraPropertyValue::Float(_) |
                    CameraPropertyValue::Double(_) |
                    CameraPropertyValue::String(_) => Some(Ordering::Greater),
                    CameraPropertyValue::Array(oa) => {
                        a.partial_cmp(oa)
                    }
                    _ => Some(Ordering::Less),
                }
            }
            CameraPropertyValue::EnumValue(_) => {
                match other {
                    CameraPropertyValue::Null |
                    CameraPropertyValue::Boolean(_) |
                    CameraPropertyValue::Integer(_) |
                    CameraPropertyValue::LongInteger(_) |
                    CameraPropertyValue::Float(_) |
                    CameraPropertyValue::Double(_) |
                    CameraPropertyValue::String(_) |
                    CameraPropertyValue::Array(_) => Some(Ordering::Greater),
                    CameraPropertyValue::EnumValue(_) => Some(Ordering::Equal),
                    _ => Some(Ordering::Less),
                }
            }
            CameraPropertyValue::Binary(b) => {
                match other {
                    CameraPropertyValue::Null|
                    CameraPropertyValue::Boolean(_)|
                    CameraPropertyValue::Integer(_)|
                    CameraPropertyValue::LongInteger(_)|
                    CameraPropertyValue::Float(_)|
                    CameraPropertyValue::Double(_)|
                    CameraPropertyValue::String(_)|
                    CameraPropertyValue::Array(_)|
                    CameraPropertyValue::EnumValue(_) => Some(Ordering::Greater),
                    CameraPropertyValue::Binary(ob) => {
                        b.partial_cmp(ob)
                    }
                    _ => Some(Ordering::Less),
                }
            }
            // FIXME: implement this lole
            CameraPropertyValue::Pair(_, _) => {
                // match other {
                //     CameraPropertyValue::Null |
                //     CameraPropertyValue::Boolean(_) |
                //     CameraPropertyValue::Integer(_) |
                //     CameraPropertyValue::LongInteger(_) |
                //     CameraPropertyValue::Float(_) |
                //     CameraPropertyValue::Double(_) |
                //     CameraPropertyValue::String(_) |
                //     CameraPropertyValue::Array(_) |
                //     CameraPropertyValue::EnumValue(_) |
                //     CameraPropertyValue::Binary(_) => Some(Ordering::Greater),
                //     CameraPropertyValue::Pair(a, b) => {
                //         match a.partial_cmp(b) {
                //             Some(_) => {}
                //             None => {}
                //         }
                //     }
                //     _ => Some(Ordering::Less)
                // }
                Some(Ordering::Equal)
            }
            CameraPropertyValue::Triple(_, _, _) => Some(Ordering::Equal),
            CameraPropertyValue::Quadruple(_, _, _, _) => Some(Ordering::Equal),
            _ => None,
        }
    }
}

impl FallibleDiv for CameraPropertyValue {
    type Output = CameraPropertyValue;
    type Error = FailedMathOp;

    fn fallible_div(&self, rhs: &Self) -> Result<Self::Output, Self::Error> {
        match self {
            CameraPropertyValue::Integer(i) => {
                if let CameraPropertyValue::Integer(rhs) = rhs {
                    return Ok(CameraPropertyValue::Integer(i / rhs));
                }
            }
            CameraPropertyValue::LongInteger(li) => {
                if let CameraPropertyValue::LongInteger(rhs) = rhs {
                    return Ok(CameraPropertyValue::LongInteger(li / rhs));
                }
            }
            CameraPropertyValue::Float(f) => {
                if let CameraPropertyValue::Float(rhs) = rhs {
                    return Ok(CameraPropertyValue::Float(f / rhs));
                }
            }
            CameraPropertyValue::Double(d) => {
                if let CameraPropertyValue::Double(rhs) = rhs {
                    return Ok(CameraPropertyValue::Double(d / rhs));
                }
            }
            _ => return Err(Default::default()),
        }
        Err(Default::default())
    }
}

impl FallibleSub for CameraPropertyValue {
    type Output = CameraPropertyValue;
    type Error = FailedMathOp;

    fn fallible_sub(&self, rhs: &Self) -> Result<Self::Output, Self::Error> {
        match self {
            CameraPropertyValue::Integer(i) => {
                if let CameraPropertyValue::Integer(rhs) = rhs {
                    return Ok(CameraPropertyValue::Integer(i - rhs));
                }
            }
            CameraPropertyValue::LongInteger(li) => {
                if let CameraPropertyValue::LongInteger(rhs) = rhs {
                    return Ok(CameraPropertyValue::LongInteger(li - rhs));
                }
            }
            CameraPropertyValue::Float(f) => {
                if let CameraPropertyValue::Float(rhs) = rhs {
                    return Ok(CameraPropertyValue::Float(f - rhs));
                }
            }
            CameraPropertyValue::Double(d) => {
                if let CameraPropertyValue::Double(rhs) = rhs {
                    return Ok(CameraPropertyValue::Double(d - rhs));
                }
            }
            _ => return Err(Self::Error::default()),
        }
        Err(Self::Error::default())
    }
}

// /// All camera controls in an array.
// #[must_use]
// pub const fn all_known_camera_controls() -> &'static [KnownCameraControl] {
//     &[
//         KnownCameraControl::Brightness,
//         KnownCameraControl::Contrast,
//         KnownCameraControl::Hue,
//         KnownCameraControl::Saturation,
//         KnownCameraControl::Sharpness,
//         KnownCameraControl::Gamma,
//         KnownCameraControl::WhiteBalance,
//         KnownCameraControl::BacklightComp,
//         KnownCameraControl::Gain,
//         KnownCameraControl::Pan,
//         KnownCameraControl::Tilt,
//         KnownCameraControl::Zoom,
//         KnownCameraControl::Exposure,
//         KnownCameraControl::Iris,
//         KnownCameraControl::Focus,
//         KnownCameraControl::Facing,
//     ]
// }

// /// The list of known camera controls to the library. <br>
// /// These can control the picture brightness, etc. <br>
// /// Note that not all backends/devices support all these. Run [`supported_camera_controls()`](crate::traits::CaptureTrait::camera_controls) to see which ones can be set.
// #[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
// #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
// pub enum KnownCameraControl {
//     Brightness,
//     Contrast,
//     Hue,
//     Saturation,
//     Sharpness,
//     Gamma,
//     WhiteBalance,
//     BacklightComp,
//     Gain,
//     Pan,
//     Tilt,
//     Zoom,
//     Exposure,
//     Iris,
//     Focus,
//     Facing,
//     /// Other camera control. Listed is the ID.
//     /// Wasteful, however is needed for a unified API across Windows, Linux, and MacOSX due to Microsoft's usage of GUIDs.
//     ///
//     /// THIS SHOULD ONLY BE USED WHEN YOU KNOW THE PLATFORM THAT YOU ARE RUNNING ON.
//     Other(u128),
// }
//
// impl Display for KnownCameraControl {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", &self)
//     }
// }
//
// /// This tells you weather a [`KnownCameraControl`] is automatically managed by the OS/Driver
// /// or manually managed by you, the programmer.
// #[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
// #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
// pub enum CameraPropertyFlag {
//     Automatic,
//     Manual,
//     Continuous,
//     ReadOnly,
//     WriteOnly,
//     Volatile,
//     Disabled,
// }
//
// impl Display for CameraPropertyFlag {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{self:?}")
//     }
// }
//
// impl ControlValueDescription {
//     /// Get the value of this [`ControlValueDescription`]
//     #[must_use]
//     pub fn value(&self) -> ControlValueSetter {
//         match self {
//             ControlValueDescription::None => ControlValueSetter::None,
//             ControlValueDescription::Integer { value, .. }
//             | ControlValueDescription::IntegerRange { value, .. } => {
//                 ControlValueSetter::Integer(*value)
//             }
//             ControlValueDescription::Float { value, .. }
//             | ControlValueDescription::FloatRange { value, .. } => {
//                 ControlValueSetter::Float(*value)
//             }
//             ControlValueDescription::Boolean { value, .. } => ControlValueSetter::Boolean(*value),
//             ControlValueDescription::String { value, .. } => {
//                 ControlValueSetter::String(value.clone())
//             }
//             ControlValueDescription::Bytes { value, .. } => {
//                 ControlValueSetter::Bytes(value.clone())
//             }
//             ControlValueDescription::KeyValuePair { key, value, .. } => {
//                 ControlValueSetter::KeyValue(*key, *value)
//             }
//             ControlValueDescription::Point { value, .. } => {
//                 ControlValueSetter::Point(value.0, value.1)
//             }
//             ControlValueDescription::Enum { value, .. } => ControlValueSetter::EnumValue(*value),
//             ControlValueDescription::RGB { value, .. } => {
//                 ControlValueSetter::RGB(value.0, value.1, value.2)
//             }
//             ControlValueDescription::StringList { value, .. } => {
//                 ControlValueSetter::StringList(value.clone())
//             }
//         }
//     }
//
//     /// Verifies if the [setter](ControlValueSetter) is valid for the provided [`ControlValueDescription`].
//     /// - `true` => Is valid.
//     /// - `false` => Is not valid.
//     ///
//     /// If the step is 0, it will automatically return `true`.
//     #[must_use]
//     pub fn verify_setter(&self, setter: &ControlValueSetter) -> bool {
//         match self {
//             ControlValueDescription::None => setter.as_none().is_some(),
//             ControlValueDescription::Integer {
//                 value,
//                 default,
//                 step,
//             } => {
//                 if *step == 0 {
//                     return true;
//                 }
//                 match setter.as_integer() {
//                     Some(i) => (i + default) % step == 0 || (i + value) % step == 0,
//                     None => false,
//                 }
//             }
//             ControlValueDescription::IntegerRange {
//                 min,
//                 max,
//                 value,
//                 step,
//                 default,
//             } => {
//                 if *step == 0 {
//                     return true;
//                 }
//                 match setter.as_integer() {
//                     Some(i) => {
//                         ((i + default) % step == 0 || (i + value) % step == 0)
//                             && i >= min
//                             && i <= max
//                     }
//                     None => false,
//                 }
//             }
//             ControlValueDescription::Float {
//                 value,
//                 default,
//                 step,
//             } => {
//                 if step.abs() == 0_f64 {
//                     return true;
//                 }
//                 match setter.as_float() {
//                     Some(f) => (f - default).abs() % step == 0_f64 || (f - value) % step == 0_f64,
//                     None => false,
//                 }
//             }
//             ControlValueDescription::FloatRange {
//                 min,
//                 max,
//                 value,
//                 step,
//                 default,
//             } => {
//                 if step.abs() == 0_f64 {
//                     return true;
//                 }
//
//                 match setter.as_float() {
//                     Some(f) => {
//                         ((f - default).abs() % step == 0_f64 || (f - value) % step == 0_f64)
//                             && f >= min
//                             && f <= max
//                     }
//                     None => false,
//                 }
//             }
//             ControlValueDescription::Boolean { .. } => setter.as_boolean().is_some(),
//             ControlValueDescription::String { .. } => setter.as_str().is_some(),
//             ControlValueDescription::Bytes { .. } => setter.as_bytes().is_some(),
//             ControlValueDescription::KeyValuePair { .. } => setter.as_key_value().is_some(),
//             ControlValueDescription::Point { .. } => match setter.as_point() {
//                 Some(pt) => {
//                     !pt.0.is_nan() && !pt.1.is_nan() && pt.0.is_finite() && pt.1.is_finite()
//                 }
//                 None => false,
//             },
//             ControlValueDescription::Enum { possible, .. } => match setter.as_enum() {
//                 Some(e) => possible.contains(e),
//                 None => false,
//             },
//             ControlValueDescription::RGB { max, .. } => match setter.as_rgb() {
//                 Some(v) => *v.0 >= max.0 && *v.1 >= max.1 && *v.2 >= max.2,
//                 None => false,
//             },
//             ControlValueDescription::StringList { availible, .. } => {
//                 availible.contains(&(setter.as_str().unwrap_or("").to_string())) // what the fuck??
//             }
//         }
//
//         // match setter {
//         //     ControlValueSetter::None => {
//         //         matches!(self, ControlValueDescription::None)
//         //     }
//         //     ControlValueSetter::Integer(i) => match self {
//         //         ControlValueDescription::Integer {
//         //             value,
//         //             default,
//         //             step,
//         //         } => (i - default).abs() % step == 0 || (i - value) % step == 0,
//         //         ControlValueDescription::IntegerRange {
//         //             min,
//         //             max,
//         //             value,
//         //             step,
//         //             default,
//         //         } => {
//         //             if value > max || value < min {
//         //                 return false;
//         //             }
//         //
//         //             (i - default) % step == 0 || (i - value) % step == 0
//         //         }
//         //         _ => false,
//         //     },
//         //     ControlValueSetter::Float(f) => match self {
//         //         ControlValueDescription::Float {
//         //             value,
//         //             default,
//         //             step,
//         //         } => (f - default).abs() % step == 0_f64 || (f - value) % step == 0_f64,
//         //         ControlValueDescription::FloatRange {
//         //             min,
//         //             max,
//         //             value,
//         //             step,
//         //             default,
//         //         } => {
//         //             if value > max || value < min {
//         //                 return false;
//         //             }
//         //
//         //             (f - default) % step == 0_f64 || (f - value) % step == 0_f64
//         //         }
//         //         _ => false,
//         //     },
//         //     ControlValueSetter::Boolean(b) => {
//         //
//         //     }
//         //     ControlValueSetter::String(_) => {
//         //         matches!(self, ControlValueDescription::String { .. })
//         //     }
//         //     ControlValueSetter::Bytes(_) => {
//         //         matches!(self, ControlValueDescription::Bytes { .. })
//         //     }
//         //     ControlValueSetter::KeyValue(_, _) => {
//         //         matches!(self, ControlValueDescription::KeyValuePair { .. })
//         //     }
//         //     ControlValueSetter::Point(_, _) => {
//         //         matches!(self, ControlValueDescription::Point { .. })
//         //     }
//         //     ControlValueSetter::EnumValue(_) => {
//         //         matches!(self, ControlValueDescription::Enum { .. })
//         //     }
//         //     ControlValueSetter::RGB(_, _, _) => {
//         //         matches!(self, ControlValueDescription::RGB { .. })
//         //     }
//         // }
//     }
// }
//
// /// This struct tells you everything about a particular [`KnownCameraControl`].
// ///
// /// However, you should never need to instantiate this struct, since its usually generated for you by `nokhwa`.
// /// The only time you should be modifying this struct is when you need to set a value and pass it back to the camera.
// /// NOTE: Assume the values for `min` and `max` as **non-inclusive**!.
// /// E.g. if the [`CameraControl`] says `min` is 100, the minimum is actually 101.
// #[derive(Clone, Debug, PartialOrd, PartialEq)]
// #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
// pub struct CameraControl {
//     control: KnownCameraControl,
//     name: String,
//     description: ControlValueDescription,
//     flag: Vec<KnownCameraControlFlag>,
//     active: bool,
// }
//
// impl CameraControl {
//     /// Creates a new [`CameraControl`]
//     #[must_use]
//     pub fn new(
//         control: KnownCameraControl,
//         name: String,
//         description: ControlValueDescription,
//         flag: Vec<KnownCameraControlFlag>,
//         active: bool,
//     ) -> Self {
//         CameraControl {
//             control,
//             name,
//             description,
//             flag,
//             active,
//         }
//     }
//
//     /// Gets the name of this [`CameraControl`]
//     #[must_use]
//     pub fn name(&self) -> &str {
//         &self.name
//     }
//
//     /// Gets the [`ControlValueDescription`] of this [`CameraControl`]
//     #[must_use]
//     pub fn description(&self) -> &ControlValueDescription {
//         &self.description
//     }
//
//     /// Gets the [`ControlValueSetter`] of the [`ControlValueDescription`] of this [`CameraControl`]
//     #[must_use]
//     pub fn value(&self) -> ControlValueSetter {
//         self.description.value()
//     }
//
//     /// Gets the [`KnownCameraControl`] of this [`CameraControl`]
//     #[must_use]
//     pub fn control(&self) -> KnownCameraControl {
//         self.control
//     }
//
//     /// Gets the [`KnownCameraControlFlag`] of this [`CameraControl`],
//     /// telling you weather this control is automatically set or manually set.
//     #[must_use]
//     pub fn flag(&self) -> &[KnownCameraControlFlag] {
//         &self.flag
//     }
//
//     /// Gets `active` of this [`CameraControl`],
//     /// telling you weather this control is currently active(in-use).
//     #[must_use]
//     pub fn active(&self) -> bool {
//         self.active
//     }
//
//     /// Gets `active` of this [`CameraControl`],
//     /// telling you weather this control is currently active(in-use).
//     pub fn set_active(&mut self, active: bool) {
//         self.active = active;
//     }
// }
//
// impl Display for CameraControl {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "Control: {}, Name: {}, Value: {}, Flag: {:?}, Active: {}",
//             self.control, self.name, self.description, self.flag, self.active
//         )
//     }
// }
//
// /// The setter for a control value
// #[derive(Clone, Debug, PartialEq, PartialOrd)]
// #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
// pub enum ControlValueSetter {
//     None,
//     Integer(i64),
//     Float(f64),
//     Boolean(bool),
//     String(String),
//     Bytes(Vec<u8>),
//     KeyValue(i128, i128),
//     Point(f64, f64),
//     EnumValue(i64),
//     RGB(f64, f64, f64),
//     StringList(String),
// }
//
// impl ControlValueSetter {
//     #[must_use]
//     pub fn as_none(&self) -> Option<()> {
//         if let ControlValueSetter::None = self {
//             Some(())
//         } else {
//             None
//         }
//     }
//     #[must_use]
//
//     pub fn as_integer(&self) -> Option<&i64> {
//         if let ControlValueSetter::Integer(i) = self {
//             Some(i)
//         } else {
//             None
//         }
//     }
//     #[must_use]
//
//     pub fn as_float(&self) -> Option<&f64> {
//         if let ControlValueSetter::Float(f) = self {
//             Some(f)
//         } else {
//             None
//         }
//     }
//     #[must_use]
//
//     pub fn as_boolean(&self) -> Option<&bool> {
//         if let ControlValueSetter::Boolean(f) = self {
//             Some(f)
//         } else {
//             None
//         }
//     }
//     #[must_use]
//
//     pub fn as_str(&self) -> Option<&str> {
//         if let ControlValueSetter::String(s) = self {
//             Some(s)
//         } else {
//             None
//         }
//     }
//     #[must_use]
//
//     pub fn as_bytes(&self) -> Option<&[u8]> {
//         if let ControlValueSetter::Bytes(b) = self {
//             Some(b)
//         } else {
//             None
//         }
//     }
//     #[must_use]
//
//     pub fn as_key_value(&self) -> Option<(&i128, &i128)> {
//         if let ControlValueSetter::KeyValue(k, v) = self {
//             Some((k, v))
//         } else {
//             None
//         }
//     }
//     #[must_use]
//
//     pub fn as_point(&self) -> Option<(&f64, &f64)> {
//         if let ControlValueSetter::Point(x, y) = self {
//             Some((x, y))
//         } else {
//             None
//         }
//     }
//     #[must_use]
//
//     pub fn as_enum(&self) -> Option<&i64> {
//         if let ControlValueSetter::EnumValue(e) = self {
//             Some(e)
//         } else {
//             None
//         }
//     }
//     #[must_use]
//
//     pub fn as_rgb(&self) -> Option<(&f64, &f64, &f64)> {
//         if let ControlValueSetter::RGB(r, g, b) = self {
//             Some((r, g, b))
//         } else {
//             None
//         }
//     }
// }
//
// impl Display for ControlValueSetter {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ControlValueSetter::None => {
//                 write!(f, "Value: None")
//             }
//             ControlValueSetter::Integer(i) => {
//                 write!(f, "IntegerValue: {i}")
//             }
//             ControlValueSetter::Float(d) => {
//                 write!(f, "FloatValue: {d}")
//             }
//             ControlValueSetter::Boolean(b) => {
//                 write!(f, "BoolValue: {b}")
//             }
//             ControlValueSetter::String(s) => {
//                 write!(f, "StrValue: {s}")
//             }
//             ControlValueSetter::Bytes(b) => {
//                 write!(f, "BytesValue: {b:x?}")
//             }
//             ControlValueSetter::KeyValue(k, v) => {
//                 write!(f, "KVValue: ({k}, {v})")
//             }
//             ControlValueSetter::Point(x, y) => {
//                 write!(f, "PointValue: ({x}, {y})")
//             }
//             ControlValueSetter::EnumValue(v) => {
//                 write!(f, "EnumValue: {v}")
//             }
//             ControlValueSetter::RGB(r, g, b) => {
//                 write!(f, "RGBValue: ({r}, {g}, {b})")
//             }
//             ControlValueSetter::StringList(s) => {
//                 write!(f, "StringListValue: {s}")
//             }
//         }
//     }
// }
//
// /// The values for a [`CameraControl`].
// ///
// /// This provides a wide range of values that can be used to control a camera.
// #[derive(Clone, Debug, PartialEq, PartialOrd)]
// #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
// pub enum ControlValueDescription {
//     None,
//     Integer {
//         value: i64,
//         default: i64,
//         step: i64,
//     },
//     IntegerRange {
//         min: i64,
//         max: i64,
//         value: i64,
//         step: i64,
//         default: i64,
//     },
//     Float {
//         value: f64,
//         default: f64,
//         step: f64,
//     },
//     FloatRange {
//         min: f64,
//         max: f64,
//         value: f64,
//         step: f64,
//         default: f64,
//     },
//     Boolean {
//         value: bool,
//         default: bool,
//     },
//     String {
//         value: String,
//         default: Option<String>,
//     },
//     Bytes {
//         value: Vec<u8>,
//         default: Vec<u8>,
//     },
//     KeyValuePair {
//         key: i128,
//         value: i128,
//         default: (i128, i128),
//     },
//     Point {
//         value: (f64, f64),
//         default: (f64, f64),
//     },
//     Enum {
//         value: i64,
//         possible: Vec<i64>,
//         default: i64,
//     },
//     RGB {
//         value: (f64, f64, f64),
//         max: (f64, f64, f64),
//         default: (f64, f64, f64),
//     },
//     StringList {
//         value: String,
//         availible: Vec<String>,
//     },
// }
//
// impl Display for ControlValueDescription {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ControlValueDescription::None => {
//                 write!(f, "(None)")
//             }
//             ControlValueDescription::Integer {
//                 value,
//                 default,
//                 step,
//             } => {
//                 write!(f, "(Current: {value}, Default: {default}, Step: {step})",)
//             }
//             ControlValueDescription::IntegerRange {
//                 min,
//                 max,
//                 value,
//                 step,
//                 default,
//             } => {
//                 write!(
//                     f,
//                     "(Current: {value}, Default: {default}, Step: {step}, Range: ({min}, {max}))",
//                 )
//             }
//             ControlValueDescription::Float {
//                 value,
//                 default,
//                 step,
//             } => {
//                 write!(f, "(Current: {value}, Default: {default}, Step: {step})",)
//             }
//             ControlValueDescription::FloatRange {
//                 min,
//                 max,
//                 value,
//                 step,
//                 default,
//             } => {
//                 write!(
//                     f,
//                     "(Current: {value}, Default: {default}, Step: {step}, Range: ({min}, {max}))",
//                 )
//             }
//             ControlValueDescription::Boolean { value, default } => {
//                 write!(f, "(Current: {value}, Default: {default})")
//             }
//             ControlValueDescription::String { value, default } => {
//                 write!(f, "(Current: {value}, Default: {default:?})")
//             }
//             ControlValueDescription::Bytes { value, default } => {
//                 write!(f, "(Current: {value:x?}, Default: {default:x?})")
//             }
//             ControlValueDescription::KeyValuePair {
//                 key,
//                 value,
//                 default,
//             } => {
//                 write!(
//                     f,
//                     "Current: ({key}, {value}), Default: ({}, {})",
//                     default.0, default.1
//                 )
//             }
//             ControlValueDescription::Point { value, default } => {
//                 write!(
//                     f,
//                     "Current: ({}, {}), Default: ({}, {})",
//                     value.0, value.1, default.0, default.1
//                 )
//             }
//             ControlValueDescription::Enum {
//                 value,
//                 possible,
//                 default,
//             } => {
//                 write!(
//                     f,
//                     "Current: {value}, Possible Values: {possible:?}, Default: {default}",
//                 )
//             }
//             ControlValueDescription::RGB {
//                 value,
//                 max,
//                 default,
//             } => {
//                 write!(
//                     f,
//                     "Current: ({}, {}, {}), Max: ({}, {}, {}), Default: ({}, {}, {})",
//                     value.0, value.1, value.2, max.0, max.1, max.2, default.0, default.1, default.2
//                 )
//             }
//             ControlValueDescription::StringList { value, availible } => {
//                 write!(f, "Current: {value}, Availible: {availible:?}")
//             }
//         }
//     }
// }

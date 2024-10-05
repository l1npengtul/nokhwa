use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    collections::{HashMap, HashSet}
};
use std::cmp::Ordering;
use crate::error::NokhwaError;
use crate::ranges::{ArrayRange, IndicatedRange, KeyValue, Options, Range, RangeValidationFailure, Simple, ValidatableRange};

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ControlValidationFailure;

impl From<RangeValidationFailure> for ControlValidationFailure {
    fn from(_: RangeValidationFailure) -> Self {
        Self
    }
}

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

    pub fn other(&self, property: &str) -> Option<&CameraPropertyDescriptor> {
        self.other.get(property)
    }

    pub fn set_other(&mut self, property: &str, value: CameraPropertyValue) -> Result<(), NokhwaError> {
        if let Some(prop) = self.other.get_mut(property) {
            prop.set_value(value)?;
            return Ok(());
        }

        Err(
            NokhwaError::SetPropertyError {
                property: property.to_string(),
                value: value.to_string(),
                error: String::from("Is null."),
            }
        )
    }
}

macro_rules! generate_property_sets {
    ( $( $name:ident, )* ) => {
        {
            impl CameraProperties {
                paste::paste! {
                $(
                pub fn [<set_ $name>](&mut self, value: CameraPropertyValue) -> Result<(), NokhwaError> {
                    if let Some(descriptor) = self.$name {
                        descriptor.set_value(value)?;
                        return Ok(())
                    }
                    return Err(
                        NokhwaError::SetPropertyError {
                            property: std::stringify!($name),
                            value: value.to_string(),
                            error: String::from("Is null."),
                        }
                    );
                }
                )*
            }
            }
        }
    };
}

generate_property_sets!( brightness, contrast, hue, saturation, sharpness, gamma, white_balance,
    backlight_compensation, gain, pan, tilt, zoom, exposure, iris, focus, facing, );

/// Describes an individual property.
#[derive(Clone, Debug)]
pub struct CameraPropertyDescriptor {
    flags: HashSet<CameraPropertyFlag>,
    platform_specific_id: Option<CameraCustomPropertyPlatformId>,
    range: CameraPropertyRange,
    value: CameraPropertyValue,
}

impl CameraPropertyDescriptor {
    pub fn new(flags: &[CameraPropertyFlag], platform_id: Option<CameraCustomPropertyPlatformId>, range: CameraPropertyRange, value: CameraPropertyValue) -> Self {
        CameraPropertyDescriptor {
            flags: HashSet::from(flags),
            platform_specific_id: platform_id,
            range,
            value,
        }
    }
    
    pub fn is_read_only(&self) -> Result<(), NokhwaError> {
        if self.flags.contains(&CameraPropertyFlag::ReadOnly) {
            return Err(NokhwaError::SetPropertyError {
                property: "Flag".to_string(),
                value: "N/A".to_string(),
                error: "Read Only".to_string(),
            })
        }
        Ok(())
    }
    
    pub fn is_write_only(&self) -> Result<(), NokhwaError> {
        if self.flags.contains(&CameraPropertyFlag::WriteOnly) {
            return Err(NokhwaError::GetPropertyError {
                property: "Flag".to_string(),
                error: "Write Only".to_string(),
            })
        }
        Ok(())
    }
    
    pub fn is_disabled(&self) -> Result<(), NokhwaError> {
        if self.flags.contains(&CameraPropertyFlag::Disabled) {
            return Err(NokhwaError::StructureError { structure: "CameraPropertyDescriptor".to_string(), error: "Disabled".to_string() })
        }
        Ok(())
    }

    pub fn flags(&self) -> Result<&HashSet<CameraPropertyFlag>, NokhwaError> {
        self.is_disabled()?;
        Ok(&self.flags)
    }
    
    pub fn platform_specific(&self) -> Option<&CameraCustomPropertyPlatformId> {
        self.platform_specific_id.as_ref()
    }

    pub fn range(&self) -> &CameraPropertyRange {
        &self.range
    }

    pub fn value(&self) -> &CameraPropertyValue {
        &self.value
    }

    pub fn set_value(&mut self, value: CameraPropertyValue) -> Result<(), NokhwaError> {
        self.range.check_value(&value)?;
        self.value = value;
        Ok(())
    }
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
#[derive(Clone, Debug)]
pub enum CameraPropertyRange {
    Null,
    Boolean(Simple<bool>),
    Integer(IndicatedRange<i64>),
    LongInteger(IndicatedRange<i128>),
    Float(IndicatedRange<f32>),
    Double(IndicatedRange<f64>),
    String(Simple<String>),
    Array(ArrayRange<Vec<CameraPropertyValue>>),
    Enumeration(Options<CameraPropertyValue>),
    Binary(Simple<Vec<u8>>),
    Pair(IndicatedRange<f32>, IndicatedRange<f32>),
    Triple(IndicatedRange<f32>, IndicatedRange<f32>, IndicatedRange<f32>),
    Quadruple(IndicatedRange<f32>, IndicatedRange<f32>, IndicatedRange<f32>, IndicatedRange<f32>),
    KeyValuePair(KeyValue<String, CameraPropertyValue>)
}

impl CameraPropertyRange {
    pub fn check_value(&self, value: &CameraPropertyValue) -> Result<(), ControlValidationFailure> {
        match self {
            CameraPropertyRange::Null => {
                if let CameraPropertyValue::Null = value {
                    return Ok(())
                }
            }
            CameraPropertyRange::Boolean(chk_b) => {
                if let CameraPropertyValue::Boolean(b) = value {
                    chk_b.validate(b)?
                }
            }
            CameraPropertyRange::Integer(chk_i) => {
                if let CameraPropertyValue::Integer(i) = value {
                    chk_i.validate(i)?
                }
            }
            CameraPropertyRange::LongInteger(chk_long) => {
                if let CameraPropertyValue::LongInteger(long) = value {
                    chk_long.validate(long)?
                }
            }
            CameraPropertyRange::Float(chk_float) => {
                if let CameraPropertyValue::Float(fl) = value {
                    chk_float.validate(fl)?;
                }
            }
            CameraPropertyRange::Double(chk_double) => {
                if let CameraPropertyValue::Double(dl) = value {
                    chk_double.validate(dl)?;
                }
            }
            CameraPropertyRange::String(chk_string) => {
                if let CameraPropertyValue::String(st) = value {
                    chk_string.validate(st)?;
                }
            }
            CameraPropertyRange::Array(chk_array) => {
                if let CameraPropertyValue::Array(arr) = value {
                    chk_array.validate(arr)?;
                }
            }
            CameraPropertyRange::Enumeration(chk_enum) => {
                if let CameraPropertyValue::EnumValue(en) = value {
                    chk_enum.validate(en)?;
                }
            }
            CameraPropertyRange::Binary(chk_bin) => {
                if let CameraPropertyValue::Binary(bin) = value {
                    chk_bin.validate(bin)?;
                }
            }
            CameraPropertyRange::Pair(chk_a, chk_b) => {
                if let CameraPropertyValue::Pair(a, b) = value {
                    chk_a.validate(a)?;
                    chk_b.validate(b)?;
                }
            }
            CameraPropertyRange::Triple(chk_x, chk_y, chk_z) => {
                if let CameraPropertyValue::Triple(x, y, z) = value {
                    chk_x.validate(x)?;
                    chk_y.validate(y)?;
                    chk_z.validate(z)?;
                }
            }
            CameraPropertyRange::Quadruple(chk_x, chk_y, chk_z, chk_w) => {
                if let CameraPropertyValue::Quadruple(x, y, z, w) = value {
                    chk_x.validate(x)?;
                    chk_y.validate(y)?;
                    chk_z.validate(z)?;
                    chk_w.validate(w)?;
                }
            }
            CameraPropertyRange::KeyValuePair(kv) => {
                if let CameraPropertyValue::KeyValue(st, va) = value {
                    if let Some(vk) = kv.by_key(st) {
                        if vk.is_same_type(va) {
                            return Ok(())
                        }
                    }
                }
            }
            _ => return Err(ControlValidationFailure),
        }
        Err(ControlValidationFailure)
    }
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
    Pair(f32, f32),
    Triple(f32, f32, f32),
    Quadruple(f32, f32, f32, f32),
    KeyValue(String, Box<CameraPropertyValue>)
}

impl CameraPropertyValue {
    pub fn is_same_type(&self, other: &CameraPropertyValue) -> bool {
        match (self, other) {
            (CameraPropertyValue::Null, CameraPropertyValue::Null) => true,
            (CameraPropertyValue::Boolean(_), CameraPropertyValue::Boolean(_)) => true,
            (CameraPropertyValue::Integer(_), CameraPropertyValue::Integer(_)) => true,
            (CameraPropertyValue::LongInteger(_), CameraPropertyValue::LongInteger(_)) => true,
            (CameraPropertyValue::Float(_), CameraPropertyValue::Float(_)) => true,
            (CameraPropertyValue::Double(_), CameraPropertyValue::Double(_)) => true,
            (CameraPropertyValue::String(_), CameraPropertyValue::String(_)) => true,
            (CameraPropertyValue::Array(_), CameraPropertyValue::Array(_)) => true,
            (CameraPropertyValue::EnumValue(_), CameraPropertyValue::EnumValue(_)) => true,
            (CameraPropertyValue::Binary(_), CameraPropertyValue::Binary(_)) => true,
            (CameraPropertyValue::Pair(..), CameraPropertyValue::Pair(..)) => true,
            (CameraPropertyValue::Triple(..), CameraPropertyValue::Triple(..)) => true,
            (CameraPropertyValue::Quadruple(..), CameraPropertyValue::Quadruple(..)) => true,
            (CameraPropertyValue::KeyValue(..), CameraPropertyValue::KeyValue(..)) => true,
            (_, _) => false,
        }
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
            CameraPropertyValue::KeyValue(k, v) => {
                if let CameraPropertyValue::KeyValue(ok, ov ) = other {
                    return (k == ok) && (v == ov)
                }
            }
            _ => {}
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

impl Display for CameraPropertyValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

use crate::error::{NokhwaError, NokhwaResult};
use crate::ranges::{Range, ValidatableRange};
use compact_str::CompactString;
use ordered_float::OrderedFloat;
use std::collections::hash_map::{Keys, Values};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hash;

pub type PlatformSpecificControlId = u64;

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum ControlId {
    FocusMode,
    FocusAutoType,
    FocusAutoRange,
    FocusAbsolute,
    FocusRelative,
    FocusStatus,

    ExposureMode,
    ExposureBias,
    ExposureMetering,
    ExposureAbsolute,
    ExposureRelative,

    IsoMode,
    IsoSensitivity,

    ApertureAbsolute,
    ApertureRelative,

    WhiteBalanceMode,
    WhiteBalanceTemperature,

    ZoomContinuous,
    ZoomRelative,
    ZoomAbsolute,

    LightingMode,
    LightingStart,
    LightingStop,
    LightingStatus,

    Orientation,

    PlatformSpecific(PlatformSpecificControlId),
}

impl Display for ControlId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Control ID: {self:?}")
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Controls {
    descriptions: HashMap<ControlId, ControlDescription>,
    values: HashMap<ControlId, ControlValue>,
}

impl Controls {
    /// INVARIANTS: All `ControlId` in `device_values` MUST exist in `device_controls`
    #[must_use]
    pub fn new(
        device_controls: HashMap<ControlId, ControlDescription>,
        device_values: HashMap<ControlId, ControlValue>,
    ) -> Option<Self> {
        for (id, value) in &device_values {
            if let Some(description) = device_controls.get(id)
                && !description.validate(value)
            {
                return None;
            }
        }

        Some(Self {
            descriptions: device_controls,
            values: device_values,
        })
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn unchecked_new(
        device_controls: HashMap<ControlId, ControlDescription>,
        device_values: HashMap<ControlId, ControlValue>,
    ) -> Self {
        Self {
            descriptions: device_controls,
            values: device_values,
        }
    }

    #[must_use]
    pub fn description(&self, control_id: &ControlId) -> Option<&ControlDescription> {
        self.descriptions.get(control_id)
    }

    #[must_use]
    pub fn value(&self, control_id: &ControlId) -> Option<&ControlValue> {
        self.values.get(control_id)
    }

    #[must_use]
    pub fn descriptions(&self) -> Values<'_, ControlId, ControlDescription> {
        self.descriptions.values()
    }

    #[must_use]
    pub fn values(&self) -> Values<'_, ControlId, ControlValue> {
        self.values.values()
    }

    #[must_use]
    pub fn ids(&self) -> Keys<'_, ControlId, ControlDescription> {
        self.descriptions.keys()
    }

    pub fn validate(
        &self,
        control_id: &ControlId,
        value: &ControlValue,
    ) -> Result<bool, NokhwaError> {
        let Some(description) = self.descriptions.get(control_id) else {
            return Err(NokhwaError::GetPropertyError {
                property: control_id.to_string(),
                error: "ID Not Found".to_string(),
            });
        };

        if !self.values.contains_key(control_id) {
            return Err(NokhwaError::GetPropertyError {
                property: control_id.to_string(),
                error: "ID Not Found".to_string(),
            });
        }

        Ok(description.validate(value))
    }

    pub fn set_control_value(
        &mut self,
        control_id: &ControlId,
        value: ControlValue,
    ) -> NokhwaResult<()> {
        match self.values.get_mut(control_id) {
            Some(old) => {
                *old = value;
                Ok(())
            }
            // this should not happen,
            None => Err(NokhwaError::SetPropertyError {
                property: control_id.to_string(),
                value: value.to_string(),
                error: "ID Not Found".to_string(),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlDescription {
    flags: HashSet<ControlFlags>,
    descriptor: ControlValueDescriptor,
    default_value: Option<ControlValue>,
}

impl ControlDescription {
    #[must_use]
    pub fn new(
        control_flags: HashSet<ControlFlags>,
        control_value_descriptor: ControlValueDescriptor,
        default_value: Option<ControlValue>,
    ) -> Option<Self> {
        if let Some(default) = &default_value
            && !control_value_descriptor.validate(default)
        {
            return None;
        }

        Some(Self {
            flags: control_flags,
            descriptor: control_value_descriptor,
            default_value,
        })
    }

    #[must_use]
    pub fn new_unchecked(
        control_flags: HashSet<ControlFlags>,
        control_value_descriptor: ControlValueDescriptor,
        default_value: Option<ControlValue>,
    ) -> Self {
        Self {
            flags: control_flags,
            descriptor: control_value_descriptor,
            default_value,
        }
    }

    #[must_use]
    pub fn flags(&self) -> &HashSet<ControlFlags> {
        &self.flags
    }

    #[must_use]
    pub fn descriptor(&self) -> &ControlValueDescriptor {
        &self.descriptor
    }

    #[must_use]
    pub fn default_value(&self) -> &Option<ControlValue> {
        &self.default_value
    }

    pub fn add_flag(&mut self, flag: ControlFlags) {
        self.flags.insert(flag);
    }

    pub fn remove_flag(&mut self, flag: ControlFlags) -> bool {
        self.flags.remove(&flag)
    }

    #[must_use]
    pub fn validate(&self, value: &ControlValue) -> bool {
        self.descriptor.validate(value)
    }
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum ControlFlags {
    Disabled,
    Busy,
    ReadOnly,
    CascadingUpdates,
    Inactive,
    Slider,
    WriteOnly,
    Volatile,
    ContinuousChange,
    ExecuteOnWrite,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControlValueDescriptor {
    Null,
    Integer(Range<i64>),
    BitMask,
    Float(Range<OrderedFloat<f64>>),
    String,
    Boolean,
    // Array of any values of singular type
    Array(Box<ControlValueDescriptor>),
    // Menu(Enum) of valid choices
    // The keys are valid choices,
    // the values represent what the choice is (usually a string or int).
    Menu(HashMap<ControlValue, ControlValue>),
    // lmao u deal with it
    // max/min length set by range. step is ALWAYS zero.
    Binary(Range<u64>),
    // An area (Resolution) like type
    Area {
        width_limits: Range<i64>,
        height_limits: Range<i64>,
    },
    // An Orientation
    // Usually, this is a read-only value.
    // An empty vec indicates any allowed value.
    Orientation(Vec<Orientation>),
}

impl ControlValueDescriptor {
    #[must_use]
    pub fn validate(&self, value: &ControlValue) -> bool {
        match self {
            ControlValueDescriptor::Null => {
                if let &ControlValue::Null = value {
                    return false;
                }
            }
            ControlValueDescriptor::Integer(int_range) => {
                if let ControlValue::Integer(i) = value {
                    return int_range.validate(i);
                }
            }
            ControlValueDescriptor::BitMask => {
                if let &ControlValue::BitMask(_) = value {
                    return true;
                }
            }
            ControlValueDescriptor::Float(float_range) => {
                if let ControlValue::Float(i) = value {
                    return float_range.validate(i);
                }
            }
            ControlValueDescriptor::String => {
                if let &ControlValue::String(_) = value {
                    return true;
                }
            }
            ControlValueDescriptor::Boolean => {
                if let &ControlValue::Boolean(_) = value {
                    return true;
                }
            }
            ControlValueDescriptor::Array(arr) => {
                if let &ControlValue::Array(_) = value {
                    return arr.validate(value);
                }
            }
            ControlValueDescriptor::Binary(size_limits) => {
                if let ControlValue::Binary(bin) = value {
                    return size_limits.validate(&(bin.len() as u64));
                }
            }
            ControlValueDescriptor::Menu(choices) => {
                if let ControlValue::EnumPick(choice) = value {
                    return choices.contains_key(choice);
                }
            }
            ControlValueDescriptor::Area {
                width_limits,
                height_limits,
            } => {
                if let ControlValue::Area { width, height } = &value {
                    return width_limits.validate(width) && height_limits.validate(height);
                }
            }
            ControlValueDescriptor::Orientation(orientations) => {
                if let ControlValue::Orientation(orientation) = &value {
                    return orientations.contains(orientation) || orientations.is_empty();
                }
            }
        }
        false
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd)]
pub enum ControlValue {
    Null,
    Integer(i64),
    BitMask(u64),
    Float(OrderedFloat<f64>),
    String(CompactString),
    Boolean(bool),
    Array(Vec<ControlValue>),
    Binary(Vec<u8>),
    EnumPick(Box<ControlValue>),
    Area { width: i64, height: i64 },
    Orientation(Orientation),
}

impl ControlValue {
    #[must_use]
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            ControlValue::Null
                | ControlValue::Integer(_)
                | ControlValue::BitMask(_)
                | ControlValue::Float(_)
                | ControlValue::String(_)
                | ControlValue::Boolean(_)
                | ControlValue::Binary(_)
                | ControlValue::Area { .. }
                | ControlValue::Orientation(_)
        )
    }

    // pub fn primitive_same_type(&self, other: &ControlValuePrimitive) -> bool {
    //     match other {
    //         ControlValuePrimitive::Null => {
    //             if let ControlValue::Null = self {
    //                 return true
    //             }
    //         }
    //         ControlValuePrimitive::Integer(_) => {if let ControlValue::Integer(_) = self {return true}}
    //         ControlValuePrimitive::BitMask(_) => {if let ControlValue::BitMask(_) = self {return true}}
    //         ControlValuePrimitive::Float(_) => {if let ControlValue::Float(_) = self {return true}}
    //         ControlValuePrimitive::String(_) => {if let ControlValue::String(_) = self {return true}}
    //         ControlValuePrimitive::Boolean(_) => {if let ControlValue::Boolean(_) = self {return true}}
    //     }
    //     false
    // }

    #[must_use]
    pub fn same_type(&self, other: &ControlValue) -> bool {
        match self {
            ControlValue::Null => {
                if let ControlValue::Null = other {
                    return true;
                }
            }
            ControlValue::Integer(_) => {
                if let ControlValue::Integer(_) = other {
                    return true;
                }
            }
            ControlValue::BitMask(_) => {
                if let ControlValue::BitMask(_) = other {
                    return true;
                }
            }
            ControlValue::Float(_) => {
                if let ControlValue::Float(_) = other {
                    return true;
                }
            }
            ControlValue::String(_) => {
                if let ControlValue::String(_) = other {
                    return true;
                }
            }
            ControlValue::Boolean(_) => {
                if let ControlValue::Boolean(_) = other {
                    return true;
                }
            }
            ControlValue::Array(_) => {
                if let ControlValue::Array(_) = other {
                    return true;
                }
            }
            ControlValue::EnumPick(_) => {
                if let ControlValue::EnumPick(_) = other {
                    return true;
                }
            }
            ControlValue::Binary(_) => {
                if let ControlValue::Binary(_) = other {
                    return true;
                }
            }
            ControlValue::Area { .. } => {
                if let ControlValue::Area { .. } = other {
                    return true;
                }
            }
            ControlValue::Orientation(_) => {
                if let ControlValue::Orientation(_) = other {
                    return true;
                }
            }
        }

        false
    }
}

impl Display for ControlValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Control Value: {self:?}")
    }
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
pub enum Orientation {
    User,
    Environment,
    Up,
    Down,
    Left,
    Right,
    Center,
    Near,
    Far,
    Other,
    Custom(i64),
}

impl Display for Orientation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Orientation {self:?}")
    }
}

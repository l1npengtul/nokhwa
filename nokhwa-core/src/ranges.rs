use std::collections::HashMap;
use core::fmt::{ Debug, Display, Formatter};
use std::collections::hash_map::Keys;
use std::hash::Hash;
use std::ops::{Div, Sub};
use crate::error::NokhwaError;

/// Failed to validate.
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct RangeValidationFailure;

/// A range type that can be validated.
pub trait ValidatableRange {
    /// Input type to validate.
    type Validation;

    /// Validates the value.
    fn validate(&self, value: &Self::Validation) -> Result<(), RangeValidationFailure>;
}

/// Creates a range of values.
///
/// Inclusive by default.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Range<T>
{
    minimum: Option<T>,
    lower_inclusive: bool,
    maximum: Option<T>,
    upper_inclusive: bool,
    preferred: T,
}

impl<T> Range<T>
where
    T: Copy + Clone + Debug + PartialOrd + PartialEq,
{
    /// Create an upper and lower inclusive [`Range`]
    pub fn new(preferred: T, min: Option<T>, max: Option<T>) -> Self {
        Self {
            minimum: min,
            lower_inclusive: true,
            maximum: max,
            upper_inclusive: true,
            preferred,
        }
    }

    pub fn with_inclusive(
        preferred: T,
        min: Option<T>,
        lower_inclusive: bool,
        max: Option<T>,
        upper_inclusive: bool,
    ) -> Self {
        Self {
            minimum: min,
            lower_inclusive,
            maximum: max,
            upper_inclusive,
            preferred,
        }
    }

    pub fn exact(preferred: T) -> Self {
        Self {
            minimum: None,
            lower_inclusive: true,
            maximum: None,
            upper_inclusive: true,
            preferred,
        }
    }

    pub fn set_minimum(&mut self, minimum: Option<T>) {
        self.minimum = minimum;
    }
    pub fn set_lower_inclusive(&mut self, lower_inclusive: bool) {
        self.lower_inclusive = lower_inclusive;
    }
    pub fn set_maximum(&mut self, maximum: Option<T>) {
        self.maximum = maximum;
    }
    pub fn set_upper_inclusive(&mut self, upper_inclusive: bool) {
        self.upper_inclusive = upper_inclusive;
    }
    pub fn set_preferred(&mut self, preferred: T) {
        self.preferred = preferred;
    }
    pub fn minimum(&self) -> Option<T> {
        self.minimum
    }
    pub fn lower_inclusive(&self) -> bool {
        self.lower_inclusive
    }
    pub fn maximum(&self) -> Option<T> {
        self.maximum
    }
    pub fn upper_inclusive(&self) -> bool {
        self.upper_inclusive
    }
    pub fn preferred(&self) -> T {
        self.preferred
    }
}

impl<T> ValidatableRange for Range<T> where T: PartialEq + PartialOrd {
    type Validation = T;

    fn validate(&self, value: Self::Validation) -> Result<(), RangeValidationFailure> {
        num_range_validate(self.minimum.as_ref(), self.maximum.as_ref(), &self.preferred, self.lower_inclusive, self.upper_inclusive, &value)
    }
}

impl<T> Default for Range<T>
where
    T: Default,
{
    fn default() -> Self {
        Range {
            minimum: None,
            lower_inclusive: true,
            maximum: None,
            upper_inclusive: true,
            preferred: T::default(),
        }
    }
}

impl<T> Display for Range<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lower_inclusive_char = bool_to_inclusive_char(self.lower_inclusive, false);
        let upper_inclusive_char = bool_to_inclusive_char(self.upper_inclusive, true);
        let default = default_to_string(&self.preferred);

        write!(f, "Range: {lower_inclusive_char}{}, {}{upper_inclusive_char}, Preferred: {default}", self.minimum, self.maximum)
    }
}


#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct IndicatedRange<T> where T: Copy + Clone + Debug + PartialOrd + PartialEq {
    minimum: T,
    lower_inclusive: bool,
    maximum: T,
    upper_inclusive: bool,
    step: Option<T>,
    default: Option<T>,
}

impl<T> IndicatedRange<T>
where
    T: Copy + Clone + Debug + PartialOrd + PartialEq
{
    pub fn new(minimum: T, lower_inclusive: bool, maximum: T, upper_inclusive: bool, step: Option<T>, default: Option<T>) -> Self {
        Self { minimum, lower_inclusive, maximum, upper_inclusive, step, default }
    }

    pub fn minimum(&self) -> T {
        self.minimum
    }

    pub fn lower_inclusive(&self) -> bool {
        self.lower_inclusive
    }

    pub fn maximum(&self) -> T {
        self.maximum
    }

    pub fn upper_inclusive(&self) -> bool {
        self.upper_inclusive
    }

    pub fn step(&self) -> Option<T> {
        self.step
    }

    pub fn default_value(&self) -> Option<T> {
        self.default
    }
}

impl<T> ValidatableRange for IndicatedRange<T> where T: Copy + PartialEq + PartialOrd + Div<Output = T> + Sub<Output = T> + Number {
    type Validation = T;

    fn validate(&self, value: &Self::Validation) -> Result<(), RangeValidationFailure> {
        if let Some(step) = &self.step {
            let prepared_value = value - &self.minimum;
            // We can check the step if we subtract the value from the minimum value
            // then see if the remainder of prepared value and step is zero.
            // e.g. 4, 12, value is 7, step is 3
            // 7 - 4 = 3
            // 3 % 3 = 0 Valid!
            if prepared_value % step != 0 {
                return Err(RangeValidationFailure::default())
            }
        }

        num_range_validate(self.minimum.as_ref(), self.maximum.as_ref(), &self.default, self.lower_inclusive, self.upper_inclusive, &value)
    }
}

impl<T> Display for IndicatedRange<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lower_inclusive_char = bool_to_inclusive_char(self.lower_inclusive, false);
        let upper_inclusive_char = bool_to_inclusive_char(self.upper_inclusive, true);
        let default = default_to_string(&self.default);
        let step = default_to_string(&self.step);

        // Ex) IndicatedRange: (5, 19], Step: 3, Default: 8
        write!(f, "IndicatedRange: {lower_inclusive_char}{}, {}{upper_inclusive_char}, Step: {step}, Default: {default}", self.minimum, self.maximum)
    }
}

#[derive(Clone, Debug)]
pub struct Options<T> where T: Clone + Debug {
    default: Option<T>,
    available: Vec<T>,
}

impl<T> Options<T>
where
    T: Clone + Debug + PartialEq
{
    pub fn new(values: Vec<T>, default_value: T) -> Self {
        Self {
            default: default_value,
            available: values,
        }
    }

    pub fn default_value(&self) -> Option<&T> {
        self.default.as_ref()
    }

    pub fn available(&self) -> &[T] {
        &self.available
    }
}

impl<T> ValidatableRange for Options<T> where T: PartialEq {
    type Validation = T;

    fn validate(&self, value: &Self::Validation) -> Result<(), RangeValidationFailure> {
        if self.available.contains(value) {
            return Ok(());
        }
        Err(RangeValidationFailure::default())
    }
}

impl<T> Display for Options<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let default = default_to_string(&self.default);

        write!(f, "Options: Available {:?}, Default: {default}", self.available)
    }
}

#[derive(Clone, Debug)]
pub struct KeyValue<K, V> where K: Clone + Debug + Hash + Eq, V: Clone + Debug {
    defaults: HashMap<K, V>,
}

impl<K, V> KeyValue<K, V>
where
    K: Clone + Debug + Hash + Eq,
    V: Clone + Debug
{
    pub fn new(default: HashMap<K, V>) -> Self {
        Self {
            defaults: default,
        }
    }

    pub fn available_keys(&self) -> &Keys<'_, K, V> {
        &self.defaults.keys()
    }
    
    pub fn by_key(&self, key: &K) -> Option<&V> {
        self.defaults.get(key)
    }
}

impl<K, V> Display for KeyValue<K, V> where K: Debug, V: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: pretty print?
        write!(f, "Key Value Pairs: {:?}", self.defaults)
    }
}

#[derive(Clone, Debug)]
pub struct ArrayRange<T> where T: Clone + Debug {
    appendable_options: Vec<T>,
    default_options: Vec<T>,
}

impl<T> ArrayRange<T> where T: Clone + Debug + PartialEq {
    pub fn new(appendable: Vec<T>, default: Vec<T>) -> Result<Self, NokhwaError> {
        for option in &default {
            if !appendable.contains(option) {
                return Err(NokhwaError::StructureError { structure: "ArrayRange".to_string(), error: "Attempted to add an undependable option to default option - ILLEGAL! - If you got this while using a driver, this is a bug! Please report to https://github.com/l1npengtul/nokhwa/issues!".to_string() })
            }
        }

        Ok(Self {
            appendable_options: appendable,
            default_options: default,
        })
    }

    pub fn appendable_options(&self) -> &[T] {
        &self.appendable_options
    }

    pub fn default_options(&self) -> &[T] {
        &self.default_options
    }
}

impl<T> ValidatableRange for ArrayRange<T> where T: PartialEq {
    type Validation = T;

    fn validate(&self, value: &Self::Validation) -> Result<(), RangeValidationFailure> {
        if self.appendable_options.contains(value) {
            return Ok(());
        }
        Err(RangeValidationFailure::default())
    }
}

impl<T> Display for ArrayRange<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArrayRange: Available Options: {:?}, Default: {:?}", self.appendable_options, self.default_options)
    }
}

#[derive(Clone, Debug)]
pub struct Simple<T> where T: Clone + Debug {
    default: Option<T>
}

impl<T> Simple<T> where T: Clone + Debug {
    pub fn new(default: Option<T>) -> Self {
        Self {
            default,
        }
    }

    pub fn default_value(&self) -> Option<&T> {
        self.default.as_ref()
    }
}

impl<T> ValidatableRange for Simple<T> {
    type Validation = T;

    fn validate(&self, _: &Self::Validation) -> Result<(), RangeValidationFailure> {
        Ok(())
    }
}

impl<T> Display for Simple<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let default = default_to_string(&self.default);
        write!(f, "Simple (Any Value): Default Value: {default}")
    }
}

fn bool_to_inclusive_char(inclusive: bool, upper: bool) -> char {
    match inclusive {
        true => if upper { ']' } else { '[' },
        false => if upper { ')' } else { '(' },
    }
}

fn default_to_string<T>(default: &Option<T>) -> String where T: Debug {
    match default {
        Some(v) => {
            format!("{v:?}")
        }
        None => String::from("None"),
    }
}


fn num_range_validate<T>(minimum: Option<&T>, maximum: Option<&T>, default: &T, lower_inclusive: bool, upper_inclusive: bool, value: &T) -> Result<(), RangeValidationFailure> where T: PartialEq + PartialOrd {
    if value == default {
        return Ok(())
    }

    if let Some(min) = minimum {
        let test = if lower_inclusive {
            min <= value
        } else {
            min < value
        };
        if test {
            return Err(RangeValidationFailure::default());
        }
    }

    if let Some(max) = maximum {
        let test = if upper_inclusive {
            max >= value
        } else {
            max > value
        };
        if test {
            return Err(RangeValidationFailure::default());
        }
    }

    Ok(())
}

trait Number {}

macro_rules! impl_num {
    ( $($n:ty, )* ) => {
        {
            $(
            impl Number for $n {}
            )*
        }
    };
}

impl_num!( i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, f32, f64, );

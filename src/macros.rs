/// Converts $from into $to
/// Example usage:
/// `tryinto_num(i32, a_unsigned_32_bit_num)`
/// Designed to deal with infallible. If not, it should be manually handled.
/// # Errors
/// If fails to convert(note: should not happen) then you messed up.
#[macro_export]
macro_rules! tryinto_num {
    ($to:ty, $from:expr) => {{
        use std::convert::TryFrom;
        match <$to>::try_from($from) {
            Ok(v) => v,
            Err(why) => {
                return Err(crate::NokhwaError::GeneralError(format!(
                    "Failed to convert {}, {}",
                    $from,
                    why.to_string()
                )))
            }
        }
    }};
}

/// Makes `init-*` functions for you. To be used in `camera.rs`
/// Example usage: check `camera.rs`
#[macro_export]
macro_rules! cap_impl_fn {
    {
        $( ($backend:ty, $init_fn:ident, $feature:expr, $backend_name:ident) ),+
    } => {
        $(
            paste::paste! {
                #[cfg(feature = $feature)]
                fn [< init_ $backend_name>](idx: usize, setting: Option<CameraFormat>) -> Option<Result<Box<dyn CaptureBackendTrait>, NokhwaError>> {
                    use crate::backends::capture::$backend;
                    match <$backend>::$init_fn(idx, setting) {
                        Ok(cap) => Some(Ok(Box::new(cap))),
                        Err(why) => Some(Err(why)),
                    }
                }
                #[cfg(not(feature = $feature))]
                fn [< init_ $backend_name>](_idx: usize, _setting: Option<CameraFormat>) -> Option<Result<Box<dyn CaptureBackendTrait>, NokhwaError>> {
                    None
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! cap_impl_matches {
    {
        $use_backend: expr, $index:expr, $setting:expr,
        $( ($feature:expr, $backend:ident, $fn:ident) ),+
    } => {
        {
            let i = $index;
            let s = $setting;
            match $use_backend {
                CaptureAPIBackend::Auto => match figure_out_auto() {
                    Some(cap) => match cap {
                        $(
                            CaptureAPIBackend::$backend => {
                                match cfg!(feature = $feature) {
                                    true => {
                                        match $fn(i,s) {
                                            Some(cap) => match cap {
                                                Ok(c) => c,
                                                Err(why) => return Err(why),
                                            }
                                            None => {
                                                return Err(NokhwaError::NotImplemented(
                                                    "Platform requirements not satisfied.".to_string(),
                                                ));
                                            }
                                        }
                                    }
                                    false => {
                                        return Err(NokhwaError::NotImplemented(
                                            "Platform requirements not satisfied.".to_string(),
                                        ));
                                    }
                                }
                            }
                        )+
                        _ => {
                            return Err(NokhwaError::NotImplemented(
                                "Platform requirements not satisfied.".to_string(),
                            ));
                        }
                    }
                    None => {
                        return Err(NokhwaError::NotImplemented(
                            "Platform requirements not satisfied.".to_string(),
                        ));
                    }
                }
                $(
                    CaptureAPIBackend::$backend => {
                        match cfg!(feature = $feature) {
                            true => {
                                match $fn(i,s) {
                                    Some(cap) => match cap {
                                        Ok(c) => c,
                                        Err(why) => return Err(why),
                                    }
                                    None => {
                                        return Err(NokhwaError::NotImplemented(
                                            "Platform requirements not satisfied.".to_string(),
                                        ));
                                    }
                                }
                            }
                            false => {
                                return Err(NokhwaError::NotImplemented(
                                    "Platform requirements not satisfied.".to_string(),
                                ));
                            }
                        }
                    }
                )+
                _ => {
                    return Err(NokhwaError::NotImplemented(
                        "Platform requirements not satisfied.".to_string(),
                    ));
                }
            }
        };
    }
}

#[cfg(feature = "input-opencv")]
#[macro_export]
macro_rules! vector {
    ( $( $elem:expr ),* ) => {
        {
            let mut vector = opencv::core::Vector::new();
            $(
                vector.push($elem);
            )*
            vector
        }
    };
}

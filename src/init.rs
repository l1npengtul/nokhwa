use crate::NokhwaError;

#[cfg(not(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
)))]
pub fn init_avfoundation(_: fn(bool)) -> Result<(), NokhwaError> {
    Ok(())
}

#[cfg(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
))]
pub fn init_avfoundation(callback: fn(bool)) -> Result<(), NokhwaError> {
    use nokhwa_bindings_macos::avfoundation::request_permission_with_callback;

    request_permission_with_callback(callback);
    Ok(())
}

#[cfg(not(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
)))]
pub fn status_avfoundation() -> bool {
    true
}

#[cfg(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
))]
pub fn status_avfoundation() -> bool {
    use nokhwa_bindings_macos::avfoundation::{
        current_authorization_status, AVAuthorizationStatus,
    };

    match current_authorization_status() {
        AVAuthorizationStatus::Authorized => true,
        _ => false,
    }
}

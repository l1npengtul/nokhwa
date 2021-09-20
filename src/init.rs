/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[cfg(not(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
)))]
fn init_avfoundation(callback: fn(bool)) {
    callback(true);
}

#[cfg(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
))]
fn init_avfoundation(callback: fn(bool)) {
    use nokhwa_bindings_macos::avfoundation::request_permission_with_callback;

    request_permission_with_callback(callback);
}

#[cfg(not(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
)))]
fn status_avfoundation() -> bool {
    true
}

#[cfg(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
))]
fn status_avfoundation() -> bool {
    use nokhwa_bindings_macos::avfoundation::{
        current_authorization_status, AVAuthorizationStatus,
    };

    matches!(
        current_authorization_status(),
        AVAuthorizationStatus::Authorized
    )
}

/// Initialize `nokhwa`
/// It is your responsibility to call this function before anything else, but only on `MacOS`.
///
/// The `on_complete` is called after initialization (a.k.a User granted permission). The callback's argument
/// is weather the initialization was successful or not
pub fn nokhwa_initialize(on_complete: fn(bool)) {
    init_avfoundation(on_complete);
}

/// Check the status if `nokhwa`
/// True if the initialization is successful (ready-to-use)
#[must_use]
pub fn nokhwa_check() -> bool {
    status_avfoundation()
}

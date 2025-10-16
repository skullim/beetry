mod check_battery;
mod drive;
mod localize;
mod read_external_data;

use beetry_backend::input;
pub use check_battery::{CheckBattery, CheckBatteryParams, CheckBatteryPlugin};
pub use drive::{Drive, DrivePlugin};
pub use localize::{Localize, LocalizePlugin};
pub use read_external_data::ReadExternalDataPlugin;

use bon::bon;

use crate::{ExternalData, Pose};

input! {DriveInput {
    pose: Pose,
}}

input! {ReadExternalDataInput {
    data: ExternalData,
}}

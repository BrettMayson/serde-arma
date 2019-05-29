use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[allow(non_snake_case)]
pub struct InternalArmaMission {
    version: u8,
    binarizationWanted: u8,
    addons: Vec<String>,
}

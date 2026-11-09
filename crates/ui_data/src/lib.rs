//! Shared APK UI schema, verified resource access and source RectTransform layout.

pub mod layout;
mod package;
pub mod sprite;
mod types;

pub use package::{UiPackage, load_ui_package};
pub use types::*;

pub type UiError = Box<dyn std::error::Error + Send + Sync>;

pub(crate) fn invalid(message: impl Into<String>) -> UiError {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()).into()
}

impl SourceReference {
    pub fn target(&self) -> Option<&midnight_map_data::SourceObject> {
        match &self.resolution {
            ReferenceState::Resolved { target, .. } => Some(target),
            _ => None,
        }
    }
}

pub mod setup;
pub mod manifest;
pub mod verify;
// pub mod utils;

pub use setup::ensure_hidden_dir;
// pub use manifest::Manifest;
pub use verify::verify_manifest;

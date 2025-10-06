pub mod setup;
pub mod manifest;
pub mod wizard;
pub mod verify;
// pub mod utils;

pub use setup::ensure_hidden_dir;
// pub use manifest::Manifest;
pub use wizard::launch_wizard;
pub use verify::verify_manifest;

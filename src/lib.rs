//! Cross-platform TCP connection metrics and loss diagnostics.

pub mod decode;
pub mod error;
pub mod model;
mod sys;
pub mod traits;

pub use error::TcpInfoError;
pub use model::{TcpMetrics, TcpState};
pub use traits::TcpInfoExt;

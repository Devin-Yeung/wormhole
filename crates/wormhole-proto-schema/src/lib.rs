mod shortcode;

pub mod shortener {
    pub mod v1 {
        tonic::include_proto!("shortener.v1");
    }
}

pub mod redirector {
    pub mod v1 {
        tonic::include_proto!("redirector.v1");
    }
}

pub mod v1 {
    pub use crate::redirector::v1::*;
    pub use crate::shortcode::v1::*;
    pub use crate::shortener::v1::*;
}

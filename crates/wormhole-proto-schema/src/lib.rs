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

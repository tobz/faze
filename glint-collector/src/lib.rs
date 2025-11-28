pub mod convert;
pub mod grpc;
pub mod http;

pub mod proto {
    pub mod opentelemetry {
        pub mod proto {
            pub mod collector {
                pub mod trace {
                    pub mod v1 {
                        tonic::include_proto!("opentelemetry.proto.collector.trace.v1");
                    }
                }
            }
            pub mod trace {
                pub mod v1 {
                    tonic::include_proto!("opentelemetry.proto.trace.v1");
                }
            }
            pub mod resource {
                pub mod v1 {
                    tonic::include_proto!("opentelemetry.proto.resource.v1");
                }
            }
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("opentelemetry.proto.common.v1");
                }
            }
        }
    }
}

pub use grpc::OtlpCollector;
pub use http::create_router;

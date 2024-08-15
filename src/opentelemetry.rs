#![macro_use]

#[allow(unused_macros)]
#[macro_export]
macro_rules! import_opentelemetry {
    () => {
        #[cfg(feature = "opentelemetry_0_24")]
        use opentelemetry_0_24_pkg as opentelemetry;

        #[cfg(feature = "opentelemetry_0_24")]
        use opentelemetry_sdk_0_24_pkg as opentelemetry_sdk;

        #[cfg(feature = "opentelemetry_0_24")]
        use opentelemetry_resourcedetector_gcp_rust_0_11_pkg as opentelemetry_resourcedetector_gcp_rust;
    };
}

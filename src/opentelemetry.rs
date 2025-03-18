#![macro_use]

#[allow(unused_macros)]
#[macro_export]
macro_rules! import_opentelemetry {
    () => {
        #[cfg(feature = "opentelemetry_0_26")]
        #[allow(unused_imports)]
        use opentelemetry_0_26_pkg as opentelemetry;

        #[cfg(feature = "opentelemetry_0_26")]
        use opentelemetry_sdk_0_26_pkg as opentelemetry_sdk;

        #[cfg(feature = "opentelemetry_0_26")]
        #[allow(unused_imports)]
        use opentelemetry_resourcedetector_gcp_rust_0_13_pkg as opentelemetry_resourcedetector_gcp_rust;

        #[cfg(feature = "opentelemetry_0_25")]
        #[allow(unused_imports)]
        use opentelemetry_0_25_pkg as opentelemetry;

        #[cfg(feature = "opentelemetry_0_25")]
        use opentelemetry_sdk_0_25_pkg as opentelemetry_sdk;

        #[cfg(feature = "opentelemetry_0_25")]
        #[allow(unused_imports)]
        use opentelemetry_resourcedetector_gcp_rust_0_12_pkg as opentelemetry_resourcedetector_gcp_rust;

        #[cfg(feature = "opentelemetry_0_24")]
        #[allow(unused_imports)]
        use opentelemetry_0_24_pkg as opentelemetry;

        #[cfg(feature = "opentelemetry_0_24")]
        use opentelemetry_sdk_0_24_pkg as opentelemetry_sdk;

        #[cfg(feature = "opentelemetry_0_24")]
        #[allow(unused_imports)]
        use opentelemetry_resourcedetector_gcp_rust_0_11_pkg as opentelemetry_resourcedetector_gcp_rust;

        #[cfg(feature = "opentelemetry_0_23")]
        #[allow(unused_imports)]
        use opentelemetry_0_23_pkg as opentelemetry;

        #[cfg(feature = "opentelemetry_0_23")]
        use opentelemetry_sdk_0_23_pkg as opentelemetry_sdk;

        #[cfg(feature = "opentelemetry_0_23")]
        #[allow(unused_imports)]
        use opentelemetry_resourcedetector_gcp_rust_0_10_pkg as opentelemetry_resourcedetector_gcp_rust;

        #[cfg(feature = "opentelemetry_0_22")]
        #[allow(unused_imports)]
        use opentelemetry_0_22_pkg as opentelemetry;

        #[cfg(feature = "opentelemetry_0_22")]
        use opentelemetry_sdk_0_22_pkg as opentelemetry_sdk;

        #[cfg(feature = "opentelemetry_0_22")]
        #[allow(unused_imports)]
        use opentelemetry_resourcedetector_gcp_rust_0_9_pkg as opentelemetry_resourcedetector_gcp_rust;

        #[cfg(feature = "opentelemetry_0_21")]
        #[allow(unused_imports)]
        use opentelemetry_0_21_pkg as opentelemetry;

        #[cfg(feature = "opentelemetry_0_21")]
        use opentelemetry_sdk_0_21_pkg as opentelemetry_sdk;

        #[cfg(feature = "opentelemetry_0_21")]
        #[allow(unused_imports)]
        use opentelemetry_resourcedetector_gcp_rust_0_8_pkg as opentelemetry_resourcedetector_gcp_rust;
    };
}

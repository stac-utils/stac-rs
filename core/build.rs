fn main() {
    #[cfg(feature = "gdal")]
    {
        use std::str::FromStr;

        // https://blog.rust-lang.org/2024/05/06/check-cfg.html
        println!("cargo::rustc-check-cfg=cfg(gdal_has_int8)");
        println!("cargo::rustc-check-cfg=cfg(gdal_has_int64)");
        println!("cargo::rustc-check-cfg=cfg(gdal_has_uint64)");

        let gdal_version_string = std::env::var("DEP_GDAL_VERSION_NUMBER").unwrap();
        let gdal_version = i64::from_str(&gdal_version_string)
            .expect("Could not convert gdal version string into number.");
        let major = gdal_version / 1000000;
        let minor = (gdal_version - major * 1000000) / 10000;
        let patch = (gdal_version - major * 1000000 - minor * 10000) / 100;

        if major != 3 {
            panic!("This crate requires a GDAL version = 3. Found {major}.{minor}.{patch}");
        }
        if minor >= 5 {
            println!("cargo:rustc-cfg=gdal_has_int64");
            println!("cargo:rustc-cfg=gdal_has_uint64");
        }
        if minor >= 7 {
            println!("cargo:rustc-cfg=gdal_has_int8");
        }
    }
}

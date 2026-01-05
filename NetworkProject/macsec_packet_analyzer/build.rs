fn main() {
    // Only link wpcap when building the CLI tool
    #[cfg(feature = "cli")]
    {
        // Point to WpdPack library directory (x64 for 64-bit compilation)
        println!("cargo:rustc-link-search=native=C:\\WpdPack\\Lib\\x64");
        println!("cargo:rustc-link-lib=wpcap");
    }
}

fn main() {
    // Link PCAP library when needed
    #[cfg(any(feature = "cli", feature = "async"))]
    {
        // Use target-specific linking
        #[cfg(target_os = "windows")]
        {
            // Point to WpdPack library directory (x64 for 64-bit compilation)
            println!("cargo:rustc-link-search=native=C:\\WpdPack\\Lib\\x64");
            println!("cargo:rustc-link-lib=wpcap");
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix-like systems (Linux, macOS), use pkg-config for libpcap
            if pkg_config::probe_library("libpcap").is_err() {
                // Fallback: assume system libpcap is available
                println!("cargo:rustc-link-lib=pcap");
            }
        }
    }
}

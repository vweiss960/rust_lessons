fn main() {
    // Tell cargo to link against the wpcap library
    println!("cargo:rustc-link-lib=wpcap");
    println!("cargo:rustc-link-lib=iphlpapi");
    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=cfgmgr32");
    println!("cargo:rustc-link-lib=ws2_32");
    println!("cargo:rustc-link-lib=ntdll");
    println!("cargo:rustc-link-lib=userenv");
    println!("cargo:rustc-link-lib=dbghelp");

    // Add the library search path
    println!("cargo:rustc-link-search=C:\\WpdPack\\Lib\\x64");

    // If you also need to link against Packet.lib (older API), uncomment:
    // println!("cargo:rustc-link-lib=Packet");
}

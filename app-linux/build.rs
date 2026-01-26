//! Build script for ViKey IBus engine
//!
//! Links with IBus and GLib libraries via pkg-config.

fn main() {
    // Link with IBus
    let ibus = pkg_config::Config::new()
        .atleast_version("1.5.0")
        .probe("ibus-1.0")
        .expect("IBus 1.5.0+ required. Install: libibus-1.0-dev (Debian) or ibus-devel (Fedora)");

    for path in &ibus.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }

    for lib in &ibus.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // Link with GLib (IBus dependency)
    let glib = pkg_config::Config::new()
        .atleast_version("2.56")
        .probe("glib-2.0")
        .expect("GLib 2.56+ required. Install: libglib2.0-dev");

    for path in &glib.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }

    for lib in &glib.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // Rerun if these change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/keymap.rs");
}

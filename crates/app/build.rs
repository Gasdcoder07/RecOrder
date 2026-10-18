fn main() {
    slint_build::compile("../../ui/main.slint").expect("failed to compile ui/main.slint");

    // Un binario sin bundle no tiene Info.plist: se embebe en la sección __info_plist
    // para que macOS conceda el permiso de micrófono y pida el de grabación con texto.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");
        println!("cargo:rustc-link-arg-bins=-Wl,-sectcreate,__TEXT,__info_plist,{manifest}/Info.plist");
        println!("cargo:rerun-if-changed=Info.plist");
    }
}

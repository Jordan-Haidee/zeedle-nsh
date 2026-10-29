fn main() {
    println!("cargo:rerun-if-changed=cover.ico");
    winresource::WindowsResource::new()
        .set_icon("cover.ico")
        .compile()
        .expect("could not embed the Zeedle icon");
}

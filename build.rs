fn main() {
    vb_rt_build::init().unwrap();

    let graphics_options = vb_graphics_build::Options::cargo_defaults().unwrap();
    vb_graphics_build::generate(graphics_options).unwrap();
}

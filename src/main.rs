use wgpuchip8::run; // Q: how do I express _this crate_ ?

fn main() {
    pollster::block_on(run());
}

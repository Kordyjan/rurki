pub fn main() {
    static TIME: f32 = 0.25;
    rig::suitest::run(TIME);
    let suite = rig::composite();
    println!("{suite:?}");
}

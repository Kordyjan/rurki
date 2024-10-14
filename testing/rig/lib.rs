#![allow(unused_mut)]
#![allow(non_snake_case)]
#![allow(clippy::module_name_repetitions)]

use engine_base::Engine;
use runner::model::Test;

pub mod input_suite;
pub mod sanity_suite;

pub fn engine_suite<T: Engine>() -> Test<T> {
    Test::Suite {
        name: "Engine tests".to_string(),
        tests: vec![sanity_suite::sanity::suite(), input_suite::input::suite()],
    }
}

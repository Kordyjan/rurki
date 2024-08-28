use std::{thread::sleep, time::Duration};

use runner::Test;

pub fn main() {
    runner::run_tests(Test::Suite {
        name: "suite 1".to_string(),
        tests: vec![
            Test::Case {
                name: "test 1".to_string(),
                code: Box::new(|| {
                    sleep(Duration::from_secs(1));
                    Err("e pyp".to_string())
                    // Ok(())
                }),
            },
            Test::Case {
                name: "test 2".to_string(),
                code: Box::new(|| {
                    sleep(Duration::from_secs(1));
                    Ok(())
                }),
            },
            Test::Suite {
                name: "suite 2".to_string(),
                tests: vec![
                    Test::Case {
                        name: "test 3".to_string(),
                        code: Box::new(|| {
                            sleep(Duration::from_secs(1));
                            Ok(())
                        }),
                    },
                    Test::Case {
                        name: "test 4".to_string(),
                        code: Box::new(|| {
                            sleep(Duration::from_secs(1));
                            Ok(())
                        }),
                    },
                ],
            },
            Test::Case {
                name: "test 5".to_string(),
                code: Box::new(|| {
                    sleep(Duration::from_secs(5));
                    Ok(())
                }),
            },
        ],
    });
}

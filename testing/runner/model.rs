use std::fmt::{write, Debug, Formatter};

use super::Result;

pub enum Test<T> {
    // Represents a single test case
    Case {
        name: String,                          // Name of the test case
        code: Box<dyn Fn(T) -> Result + Send>, // Closure containing the test logic
    },
    // Represents a group of tests (test suite)
    Suite {
        name: String,        // Name of the test suite
        tests: Vec<Test<T>>, // Vector of child tests (can be Cases or nested Suites)
    },
}

impl<T> Test<T> {
    fn write(&self, f: &mut Formatter, prefix: &str, child_prefix: &str) {
        match self {
            Test::Case { name, .. } => {
                writeln!(f, "{}{}", prefix, name).unwrap();
            }
            Test::Suite { name, tests } => {
                writeln!(f, "{}{}", prefix, name).unwrap();
                if !tests.is_empty() {
                    let last_n = tests.len() - 1;
                    let last = &tests[last_n];
                    let rest = &tests[..last_n];
                    for test in rest {
                        test.write(f, &format!("{}├─ ", child_prefix), &"│  ".to_string());
                    }
                    last.write(f, &format!("{}└─ ", child_prefix), &"   ".to_string());
                }
            }
        }
    }
}

impl<T> Debug for Test<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.write(f, "", "");
        Ok(())
    }
}

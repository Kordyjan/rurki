use rig_macros::test_suite;

#[test_suite]
pub mod suite {
    pub fn something_other() {
        println!("not a test");
    }

    #[case]
    pub fn testing_1() {
        println!("test1");
    }

    #[case]
    pub fn testing_2() {
        println!("test2");
    }
}

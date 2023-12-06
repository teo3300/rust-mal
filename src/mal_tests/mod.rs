#[cfg(test)]
mod functional {

    macro_rules! test {
        ($file:expr) => {{
            use crate::core::ns_init;
            use crate::load_file;
            assert!(matches!(
                load_file(format!("tests/{}.mal", $file).as_str(), &ns_init()),
                Ok(_)
            ));
        }};
    }

    #[test]
    fn assert_fail() {
        use crate::core::ns_init;
        use crate::load_file;
        assert!(matches!(
            load_file("tests/assert_fail.mal", &ns_init()),
            Err(_)
        ))
    }

    #[test]
    fn builtin_logic() {
        test!("logic")
    }

    #[test]
    fn builtin_equals() {
        test!("equals");
    }

    #[test]
    fn arithmetic() {
        test!("arithmetic")
    }

    #[test]
    fn fibonacci() {
        test!("fibonacci");
    }
}

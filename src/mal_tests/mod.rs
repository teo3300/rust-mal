#[cfg(test)]
mod functional {

    macro_rules! test {
        ($file:expr) => {{
            use crate::core::ns_init;
            use crate::load_file;
            use crate::parse_tools::{load_home_file, set_home_path};
            let env = ns_init();
            set_home_path(&env);
            load_home_file("core.mal", &env, false);
            assert!(matches!(
                load_file(format!("tests/{}.mal", $file).as_str(), &env),
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

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
    // TODO: modify to accept more parameters for test/libraries
    // TODO: text 'boom' from within rust

    #[test]
    fn assert() {
        test!("assert")
    }

    #[test]
    fn builtin_logic() {
        test!("logic")
    }

    #[test]
    fn builtin_equals() {
        test!("equals")
    }

    #[test]
    fn arithmetic() {
        test!("arithmetic")
    }

    #[test]
    fn fibonacci() {
        test!("fibonacci")
    }

    #[test]
    fn forms() {
        test!("forms")
    }

    #[test]
    fn lists() {
        test!("lists")
    }

    #[test]
    fn atoms() {
        test!("atoms")
    }

    #[test]
    fn car_cdr() {
        test!("car-cdr")
    }

    #[test]
    fn map() {
        test!("map")
    }

    #[test]
    fn fil() {
        test!("fil")
    }
}

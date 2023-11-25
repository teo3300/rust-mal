#[cfg(test)]
mod functional {

    macro_rules! load_file {
        ($file:expr, $env:expr) => {{
            match load_file($file, $env) {
                Ok(v) => v,
                Err(_) => {
                    panic!()
                }
            }
        }};
    }

    macro_rules! test {
        ($file:expr) => {{
            use crate::core::ns_init;
            use crate::load_file;
            let env = ns_init();
            load_file!("core.mal", &env);
            load_file!(format!("tests/{}.mal", $file).as_str(), &env);
        }};
    }

    #[test]
    fn fibonacci() {
        test!("fibonacci");
    }

    #[test]
    fn builtin_equals() {
        test!("equals");
    }
}

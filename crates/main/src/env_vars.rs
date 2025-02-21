//! TODO
//! It would be nice to also declare optional types and default values:
//!
//!     env_vars!(
//!       ARCHIVE_DIR std::path::PatBuf,
//!       TIMEOUT u32 "60000",
//!       BOT_NAME,
//!       BOT_URL "https://..."
//!     );
//!
//! But I did not manage to write the macro this way.
//! <https://stackoverflow.com/questions/76567540/rust-macro-count-repetitions-of-nested-match>

// source: https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html
pub const fn count_helper<const N: usize>(_: [(); N]) -> usize {
    N
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! count_tts {
    ($($smth:tt)*) => {
        $crate::env_vars::count_helper([$(replace_expr!($smth ())),*])
    }
}

macro_rules! evwc {
    ( $cnt:expr; $($name:ident)+ ) => {
        mod env_config {
            #[derive(Debug)]
            pub struct EnvConfig<'a> {
                pub name: &'a str
            }

            impl EnvConfig<'_> {
                pub fn get(&self) -> String {
                    match std::env::var(self.name) {
                        Err(e) => {panic!("Error when getting environment variable {}: {:?} ", self.name, e)}
                        Ok(v) => v
                    }
                }

                pub fn parse<T>(&self) -> T
                where
                    T: std::str::FromStr,
                    T::Err: std::fmt::Display,
                {
                    let value = self.get();
                    match value.parse() {
                        Err(e) => {
                            panic!(
                                "Error when parsing environment variable {} to type {}. {}. Value: {:?} ",
                                self.name,
                                std::any::type_name::<T>(),
                                e,
                                value
                            )
                        }
                        Ok(v) => v
                    }
                }
            }

            $(pub const $name: EnvConfig =
              EnvConfig{
                  name: stringify!($name)
              }; ) +

            pub fn all_vars() -> [EnvConfig<'static>; $cnt] {
                [$($name,) +]
            }

            pub fn get_missing_iter() -> impl Iterator<Item = &'static str>{
                all_vars().into_iter().filter(
                    |x| std::env::var(x.name).is_err()
                ).map(|x| x.name)
            }

            pub fn get_missing() -> Vec<&'static str> {
                get_missing_iter().collect()
            }

            pub fn check() {
                let missing = get_missing();
                if !missing.is_empty() {
                    panic!("Missing environment variables: {:?}", missing);
                }
            }

            pub fn get_map() -> std::collections::HashMap<String, String> {
                let mut map = std::collections::HashMap::new();
                for var in all_vars() {
                    map.insert(var.name.to_string(), var.get());
                }
                map
            }
        }
    }
}

macro_rules! env_vars {
  ( $($env_var:tt)+ ) => (evwc!(count_tts!($($env_var)+); $($env_var)+ );)
}

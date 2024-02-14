use serde::{self, de::DeserializeOwned};
use std::fs;

pub trait Config<Optional: ConfigOptional> {
    fn create_from_optional(optional: Optional) -> Self;
}

pub trait ConfigOptional {
    fn create() -> Self;
}

pub fn create_config<ConcreteConfig, OptionalConfig>(
    config_file_name: &'static str,
    config_path: &'static str,
    default_config: String,
) -> ConcreteConfig
where
    ConcreteConfig: Config<OptionalConfig>,
    OptionalConfig: ConfigOptional + DeserializeOwned,
{
    let base = directories_next::BaseDirs::new().unwrap();
    let home_dir = base.config_dir();
    if !home_dir.is_dir() {
        panic!("There is no home directory, please ensure your PC has a home directory.");
    }
    let config_dir = home_dir.join(config_path);
    if !config_dir.is_dir() {
        fs::create_dir(&config_dir).expect("Could not create config folder");
    }
    let config_file = &config_dir.join(config_file_name);
    if !config_file.is_file() {
        fs::File::create(config_file).expect("Could not create config file");
    }
    let contents = match fs::read_to_string(config_file) {
        Ok(c) => c,
        Err(_) => default_config,
    };
    let parsed_conf: OptionalConfig = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => toml::from_str(&contents).unwrap(),
    };
    ConcreteConfig::create_from_optional(parsed_conf)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{create_config, Config, ConfigOptional};

    #[derive(Debug, Deserialize)]
    struct Conf {
        something: u32,
        what: String,
    }

    impl Config<OptConf> for Conf {
        fn create_from_optional(optional: OptConf) -> Conf {
            let something = if let Some(something) = optional.something {
                something
            } else {
                0
            };
            let what = if let Some(what) = optional.what {
                what
            } else {
                String::from("pingpang")
            };
            Conf { something, what }
        }
    }

    #[derive(Debug, Deserialize)]
    struct OptConf {
        something: Option<u32>,
        what: Option<String>,
    }

    impl ConfigOptional for OptConf {
        // TODO: the last piece would be the creation of a macro for this
        fn create() -> Self {
            OptConf {
                something: None,
                what: Some(String::from("grengeng")),
            }
        }
    }

    #[test]
    fn config_test() {
        let conf =
            create_config::<Conf, OptConf>("config.toml", "oxilib", format!("r#something = 10#"));
        dbg!(&conf);
        assert_eq!(conf.something, 10);
        assert_eq!(conf.what, String::from("pingpang"));
    }
}

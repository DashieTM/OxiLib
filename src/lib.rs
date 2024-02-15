use serde::{self, de::DeserializeOwned};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug)]
pub struct ReadConfigFileError {}
impl Display for ReadConfigFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Error on reading File.")
    }
}
impl std::error::Error for ReadConfigFileError {}

pub trait Config<Optional: ConfigOptional> {
    fn create_from_optional(optional: Optional) -> Self;
}

pub trait ConfigOptional {
    fn create() -> Self;
}

pub fn create_config_folder(config_path: &'static str) -> PathBuf {
    let base = directories_next::BaseDirs::new().unwrap();
    let home_dir = base.config_dir();
    if !home_dir.is_dir() {
        panic!("There is no home directory, please ensure your PC has a home directory.");
    }
    let config_dir = home_dir.join(config_path);
    if !config_dir.is_dir() {
        fs::create_dir(&config_dir).expect("Could not create config folder");
    }
    config_dir
}

pub fn read_specific_file(absolute_path: &'static str) -> Result<String, ReadConfigFileError> {
    let path = PathBuf::from_str(absolute_path);
    if path.is_err() {
        return Err(ReadConfigFileError {});
    }
    let path = path.unwrap();
    if !path.is_file() {
        return Err(ReadConfigFileError {});
    }

    let content = fs::read_to_string(path);
    if content.is_err() {
        return Err(ReadConfigFileError {});
    }

    Ok(content.unwrap())
}

pub fn create_config<ConcreteConfig, OptionalConfig>(
    config_dir: &Path,
    config_file_name: &'static str,
    default_config: &'static str,
) -> ConcreteConfig
where
    ConcreteConfig: Config<OptionalConfig>,
    OptionalConfig: ConfigOptional + DeserializeOwned,
{
    let config_file = config_dir.join(config_file_name);
    if !config_file.is_file() {
        fs::File::create(&config_file).expect("Could not create config file");
    }
    let contents = match fs::read_to_string(config_file) {
        Ok(c) => {
            if c.is_empty() {
                default_config.to_string()
            } else {
                c
            }
        }
        Err(_) => default_config.to_string(),
    };
    let parsed_conf: OptionalConfig = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => toml::from_str(&contents).unwrap(),
    };
    ConcreteConfig::create_from_optional(parsed_conf)
}

pub fn create_css(config_dir: &Path, css_file: &'static str, css_content: &'static str) -> PathBuf {
    let css_file = config_dir.join(css_file);
    if !css_file.is_file() {
        fs::File::create(&css_file).expect("Could not create css file.");
    }
    if fs::read(&css_file)
        .expect("Could check css file content")
        .is_empty()
    {
        fs::write(&css_file, css_content).expect("Could not write default css content.");
    }
    css_file
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use serde::Deserialize;

    use crate::{create_config, create_config_folder, create_css, Config, ConfigOptional};

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
    fn test_config_folder() {
        let config_dir = create_config_folder("testfolder");
        assert!(&config_dir.is_dir());
        fs::remove_dir(&config_dir).expect("Could not remove testfolder again.");
    }

    #[test]
    fn test_config() {
        let config_dir = create_config_folder("testfolder2");
        let conf = create_config::<Conf, OptConf>(&config_dir, "config.toml", "something = 10");
        assert_eq!(conf.something, 10);
        assert_eq!(conf.what, String::from("pingpang"));
        fs::remove_dir_all(&config_dir).expect("Could not remove testfolder again.");
    }

    #[test]
    fn test_css() {
        let config_dir = create_config_folder("testfolder3");
        let css_content = ".something {
            color: red;
        }";
        let css = create_css(&config_dir, "style.css", css_content);
        let read_css = fs::read_to_string(css).expect("Could not read created css file.");
        assert_eq!(css_content, read_css);
        fs::remove_dir_all(&config_dir).expect("Could not remove testfolder again.");
    }

    #[test]
    fn test_custom_file() {
        let mut file = fs::File::create("test.txt").expect("Could not create test file");
        let content = "this is a random test";
        file.write_all(content.as_bytes())
            .expect("Could not write to test file.");
        let read_content =
            fs::read_to_string("test.txt").expect("Could not read from created file.");
        assert_eq!(content, &read_content);
        fs::remove_file("test.txt").expect("Could not remove testfolder again.");
    }
}

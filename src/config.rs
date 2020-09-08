use crate::{AError, Configuration, MenuConfig, MenuSystem, Result};
use regex::Regex;
use std::fs::File;
use std::{collections::HashMap, io::prelude::*, path::PathBuf};

pub const EXAMPLE_CONFIG: &str = "
# this should go into ~/.config/auswahl/auswahlrc
use: rofi -dmenu

set-redshift
  off => redshift -x
  medium => redshift -P -O 5000
  high => redshift -P -O 4500

search :: -i -a
  g => firefox https://google.com?q={{}}
  yt => firefox https://youtube.com/results?search_query={{}}
";

pub fn read_config_file(filepath: Option<PathBuf>) -> Result<Configuration> {
    let config_file_path = filepath.unwrap_or_else(|| {
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from(std::env::var("HOME").unwrap()).join(".config"))
            .join("auswahl/auswahlrc")
    });
    let mut file_content = String::new();
    File::open(config_file_path)?.read_to_string(&mut file_content)?;
    parse_config(&file_content)
}

pub fn parse_config(s: &str) -> Result<Configuration> {
    let mut menus: HashMap<String, MenuConfig> = HashMap::new();
    let mut cur_menu: Option<MenuConfig> = None;
    let mut menu_system: Option<MenuSystem> = None;
    for line in s.lines().filter(|x| !x.starts_with('#')) {
        if line.starts_with("use:") {
            menu_system = Regex::new(r"^use:\s*(.+?)$")
                .unwrap()
                .captures(line)
                .ok_or(AError::Parse(format!("Error parsing use: \"{}\"", line)))?
                .get(1)
                .map(|x| MenuSystem(x.as_str().to_string()));
        } else if line.starts_with("menu:") {
            cur_menu.take().map(|menu| menus.insert(menu.menu_name.clone(), menu));
            let (menu_name, additional_flags) = parse_menu_title(line)?;
            cur_menu = Some(MenuConfig {
                menu_name,
                additional_flags,
                options: Vec::new(),
            });
        } else if line.starts_with(" ") {
            let (option_key, option_value) = parse_menu_option(line.trim())?;
            cur_menu.as_mut().map(|menu| {
                menu.options.push((option_key.to_string(), option_value));
            });
        } else if line.trim().is_empty() {
            cur_menu.take().map(|menu| menus.insert(menu.menu_name.clone(), menu));
        }
    }
    cur_menu.take().map(|menu| menus.insert(menu.menu_name.clone(), menu));
    Ok(Configuration {
        menu_system: menu_system.ok_or_else(|| AError::Parse("No menu system configured".to_string()))?,
        menus,
    })
}

fn parse_menu_title(s: &str) -> Result<(String, String)> {
    let captures = Regex::new(r"^menu:\s*(.*)\s*$").unwrap().captures(s);
    let content: &str = &captures.ok_or(AError::Parse(format!("Error parsing menu title \"{}\"", s)))?[1];

    // check for commandline flags passed in menu title
    let flags_regex = Regex::new(r"^(.*?)\s*::\s*(.*)$").unwrap();
    if flags_regex.is_match(content) {
        let captures = flags_regex.captures(content).unwrap();
        let title = captures[1].to_string();
        let flags = captures[2].to_string();
        Ok((title, flags))
    } else {
        Ok((content.to_string(), String::new()))
    }
}

fn parse_menu_option(s: &str) -> Result<(String, String)> {
    Regex::new(r"^\s*(.*?)\s*=>\s*(.*?)\s*$")
        .unwrap()
        .captures(s)
        .ok_or(AError::Parse(format!("could not parse option \"{}\"", s)))
        .map(|matches| (matches[1].to_string(), matches[2].to_string()))
}

#[cfg(test)]
mod test {
    use crate::config::*;
    use maplit::hashmap;
    #[test]
    fn test_parse_menu_option() {
        assert_eq!(
            parse_menu_option("off => setthing off"),
            Ok(("off".to_string(), "setthing off".to_string())),
        );
    }

    #[test]
    fn test_parser() {
        let parsed = parse_config(EXAMPLE_CONFIG);
        let expected_menus = hashmap! {
            "set-redshift".to_string() => MenuConfig {
                menu_name: "set-redshift".to_string(),
                additional_flags: String::new(),
                options: vec! [
                    ("off".to_string(), "redshift -x".to_string()),
                    ("on".to_string() ,"redshift -O 4500".to_string())
                ]
            },
            "search".to_string() => MenuConfig {
                menu_name: "search".to_string(),
                additional_flags: "-i -a".to_string(),
                options: vec![
                    ("g {}".to_string(), "firefox https://google.com?q=$1".to_string()),
                    ("yt {}".to_string(), "firefox https://youtube.com?q=$1".to_string())
                ]
            }
        };
        let expected = Configuration {
            menu_system: MenuSystem("rofi -dmenu".to_string()),
            menus: expected_menus,
        };
        assert_eq!(Ok(expected), parsed);
    }
}

use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;
use std::{
    collections::HashMap,
    process::{Command, Stdio},
};
use structopt::StructOpt;

extern crate maplit;
pub mod config;

type Result<T> = std::result::Result<T, AError>;

#[derive(Debug, PartialEq, Eq)]
pub enum AError {
    Parse(String),
    IO(String),
    CliArg(String),
    MenuSystem(String),
    NothingSelected,
    UnknownMenu,
}

impl std::fmt::Display for AError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AError::Parse(msg) => format!("Parse error:\n{}", msg),
            AError::IO(msg) => format!("IO error:\n{}", msg),
            AError::CliArg(msg) => format!("Error while reading command line arguments:\n{}", msg),
            AError::MenuSystem(msg) => format!("Error while running the menu:\n{}", msg),
            AError::NothingSelected => "".to_string(),
            AError::UnknownMenu => "Requested menu not found".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl From<std::io::Error> for AError {
    fn from(e: std::io::Error) -> Self {
        AError::IO(e.to_string())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "auswahl")]
struct Opt {
    #[structopt(short, long)]
    list: bool,

    #[structopt(short, long = "--config", parse(from_os_str))]
    config_file: Option<PathBuf>,

    #[structopt(long = "--example")]
    print_example_config: bool,

    requested_menu: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct MenuSystem(String);
impl MenuSystem {
    fn prompt_menu(&self, flags: &str, options: Vec<&str>) -> Result<String> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(self.0.to_string() + " " + flags)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all((options.join("\n") + "\n").as_bytes())?;
        std::mem::drop(stdin);
        let selection = std::io::BufReader::new(child.stdout.take().unwrap())
            .lines()
            .collect::<std::io::Result<Vec<String>>>()?
            .join("\n");
        Ok(selection)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MenuConfig {
    menu_name: String,
    additional_flags: String,
    options: Vec<(String, String)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Configuration {
    menus: HashMap<String, MenuConfig>,
    menu_system: MenuSystem,
}

fn main() {
    let result = run_program();
    match result {
        Err(err) => println!("{}", err),
        Ok(()) => {}
    }
}

fn run_program() -> Result<()> {
    let opt = Opt::from_args();
    let config = config::read_config_file(opt.config_file)?;
    if opt.list {
        config.menus.keys().for_each(|menu| println!("{}", menu));
    } else if opt.print_example_config {
        println!("{}", config::EXAMPLE_CONFIG);
    } else {
        match opt.requested_menu {
            Some(requested_menu) => run(&config, &requested_menu)?,
            None => {}
        }
    }
    Ok(())
}

fn run(config: &Configuration, requested_menu: &str) -> Result<()> {
    let requested_menu = config.menus.get(requested_menu).ok_or(AError::UnknownMenu)?;
    let selection = config.menu_system.prompt_menu(
        &requested_menu.additional_flags,
        requested_menu.options.iter().map(|(key, _)| key.as_ref()).collect(),
    )?;
    let selected_entry = requested_menu
        .options
        .iter()
        .find(|(key, _)| selection.starts_with(key))
        .ok_or(AError::NothingSelected)?;
    let command_with_arg = selected_entry
        .1
        .replace("{{}}", selection.trim_start_matches(&selected_entry.0).trim());
    Command::new("bash").arg("-c").arg(command_with_arg).spawn()?;
    Ok(())
}

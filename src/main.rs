extern crate clap;
extern crate java_properties;

use clap::{Arg, ArgMatches, App, SubCommand};
use std::io::Write;
use std::ffi::OsString;

use result::*;

pub mod result;
pub mod fs;
pub mod os;

fn main() {
    let mut stderr = std::io::stderr();

    let matches = App::new("rsenv")
        .version("1.0")
        .author("Chris Dawes <cmsd2@cantab.net>")
        .about("Manages shell environments")
        .subcommand(SubCommand::with_name("list")
                    .about("Lists available shell environments")
                    .author("Chris Dawes <cmsd2@cantab.net>")
                    .version("1.0")
                    )
        .subcommand(SubCommand::with_name("remove")
                    .about("Removes installed environment from global environments dir")
                    .author("Chris Dawes <cmsd2@cantab.net>")
                    .version("1.0")
                    .arg(Arg::with_name("name")
                         .short("n")
                         .required(true)
                         .takes_value(true)
                         .index(1)
                         .help("name of environment to remove")
                         )
                    )
        .subcommand(SubCommand::with_name("show")
                    .about("Shows installed environment")
                    .author("Chris Dawes <cmsd2@cantab.net>")
                    .version("1.0")
                    .arg(Arg::with_name("name")
                         .short("n")
                         .required(true)
                         .takes_value(true)
                         .index(1)
                         .help("name of environment to show")
                         )
                    )
        .subcommand(SubCommand::with_name("edit")
                    .about("Starts editor for installed environment")
                    .author("Chris Dawes <cmsd2@cantab.net>")
                    .version("1.0")
                    .arg(Arg::with_name("name")
                         .short("n")
                         .required(true)
                         .takes_value(true)
                         .index(1)
                         .help("name of environment to edit")
                         )
                    )
        .subcommand(SubCommand::with_name("install")
                    .about("Installs env file in global environments dir")
                    .author("Chris Dawes <cmsd2@cantab.net>")
                    .version("1.0")
                    .arg(Arg::with_name("name")
                         .short("n")
                         .required(true)
                         .takes_value(true)
                         .index(1)
                         .help("name of environment being installed")
                         )
                    .arg(Arg::with_name("file")
                         .short("f")
                         .required(true)
                         .takes_value(true)
                         .index(2)
                         .help("the environment file to install")
                         )
                    )
        .subcommand(SubCommand::with_name("exec")
                    .about("Exec a command inside an environment")
                    .author("Chris Dawes <cmsd2@cantab.net>")
                    .version("1.0")
                    .arg(Arg::with_name("name")
                         .short("n")
                         .required(true)
                         .takes_value(true)
                         .index(1)
                         .help("name of environment to load")
                         )
                    .arg(Arg::with_name("command")
                         .short("c")
                         .required(true)
                         .takes_value(true)
                         .index(2)
                         .multiple(true)
                         .help("shell command to run")
                         )
                    )
        .get_matches();

    match run_subcommand(&matches) {
        Err(err) => {
            writeln!(&mut stderr, "Error: {:?}", err).unwrap();
        },
        _ => {}
    }
}

fn run_subcommand(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("list", Some(_)) => list_envs(),
        ("install", Some(sub_matches)) => install_env(sub_matches),
        ("remove", Some(sub_matches)) => remove_env(sub_matches),
        ("exec", Some(sub_matches)) => exec_command(sub_matches),
        ("show", Some(sub_matches)) => show_env(sub_matches),
        ("edit", Some(sub_matches)) => edit_env(sub_matches),
        _ => Ok(())
    }
}

fn list_envs() -> Result<()> {
    try!(init_dirs());
    
    for e in try!(get_envs_list()) {
        println!("{}", e);
    }

    Ok(())
}

fn install_env(args: &ArgMatches) -> Result<()> {
    let file_name = args.value_of("file").unwrap();
    let env_name = args.value_of("name").unwrap();

    let file_path = std::path::Path::new(file_name);
    try!(fs::assert_file_exists(&file_path));
    
    let dest_file_path = try!(fs::get_installed_env_file(env_name));

    try!(std::fs::copy(file_path, dest_file_path.as_path()));

    Ok(())
}

fn remove_env(args: &ArgMatches) -> Result<()> {
    let env_name = args.value_of("name").unwrap();

    let dest_file_path = try!(fs::get_installed_env_file(env_name));
    try!(fs::assert_file_exists(&dest_file_path));

    try!(std::fs::remove_file(dest_file_path));

    Ok(())
}

fn show_env(args: &ArgMatches) -> Result<()> {
    let env_name = args.value_of("name").unwrap();

    let env = try!(fs::load_installed_env_file(env_name));

    print_env(&env);

    Ok(())
}

fn edit_env(args: &ArgMatches) -> Result<()> {
    let env_name = args.value_of("name").unwrap();

    try!(fs::edit_installed_env_file(env_name));

    Ok(())
}

fn exec_command(args: &ArgMatches) -> Result<()> {
    let command_line: Vec<&str> = args.values_of("command").unwrap().collect();
    let env_name = args.value_of("name").unwrap();

    let env = try!(fs::load_installed_env_file(env_name));
    
    let mut command_line_iter = command_line.into_iter();
    let command_name = command_line_iter.next().unwrap();
    let args: Vec<&str> = command_line_iter.collect();

    fs::spawn_command(OsString::from(command_name).as_os_str(), &args[..], &env)
}

fn file_name_to_env_name<'a>(file_name: &'a str) -> Option<&'a str> {
    let split: Vec<&'a str> = file_name
        .rsplitn(2, ".env")
        .collect();

    if split.len() == 2 {
        split.get(1).map(|o| *o)
    } else {
        None
    }
}

fn get_envs_list() -> Result<Vec<String>> {
    let env_file_list = try!(fs::list_env_files());

    let maybe_env_list = env_file_list
        .iter()
        .flat_map(|file_name| file_name_to_env_name(file_name))
        .map(|s| s.to_owned())
        .collect();
    
    Ok(maybe_env_list)
}

fn print_env(env: &fs::RsEnv) {
    for (k,v) in &env.vars {
        println!("{} = {}", k, v);
    }
}

fn init_dirs() -> Result<()> {
    try!(os::get_config_dir());
    
    Ok(())
}

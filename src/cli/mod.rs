use clap::{App, Arg, ArgMatches};

pub const DEFAULT_FILE_NAME: &'static str = "package.json";

///
/// Function constructs the parser for `tows` command
///
pub fn build<'a>() -> ArgMatches<'a> {
  let cwd = Arg::with_name("cwd")
    .short("C")
    .long("cwd")
    .value_name("CWD")
    .help("Sets a custom cwd")
    .takes_value(true);

  let filename = Arg::with_name("filename")
    .short("f")
    .long("filename")
    .default_value(DEFAULT_FILE_NAME)
    .help("Sets a custom filename (default: package.json)")
    .takes_value(true);

  App::new("tows")
    .version("0.1")
    .author("nju33 <nju33.ki@gmail.com>")
    .about("todo")
    .arg(cwd)
    .arg(filename)
    .get_matches()
}

use clap::{Arg, App};

fn main() {
    let matches = App::new("confidence").version("0.1.0")
            .author("John Baber-Lucero <cargo@frundle.com>")
            .about("Given one directory, output file full of hashes to compare with a future
directory, or compare a given directory to a provided file full of hashes.\nGiven two directories, directly compare the directories' files.")
            .arg(Arg::with_name("ignore-permission-errors")
                    .short("i")
                    .long("ignore-permission-errors")
                    .help("Ignore errors so you can skip files you don't have permission to read. Useful for examining everything on a drive that your non-root user can see.")
                    .takes_value(false)
            ).arg(Arg::with_name("verbosity")
                    .short("v")
                    .multiple(true)
                    .help("verbosity level (0 - 3 v's)")
            ).arg(Arg::with_name("output")
                    .short("o")
                    .long("output-filename")
                    .takes_value(true)
                    .help("File to output hashes to (required if only <directory_one> provided)")
            ).arg(Arg::with_name("size")
                    .help("<directory_one>'s approximate size in bytes")
                    .required(true)
                    .index(1)
            ).arg(Arg::with_name("directory_one")
                    .required(true)
                    .index(2)
            ).arg(Arg::with_name("directory_two")
                    .help("If present, we'll just directly compare <directory_one> and <directory_two>")
                    .required(false)
                    .conflicts_with("output")
                    .index(3)
            ).get_matches();


    std::process::exit(confidence::actual_runtime(matches));
}

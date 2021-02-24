use docopt::Docopt;
use walkdir::WalkDir;

const VERSION: &str = "0.1.0";

const USAGE: &str = "Usage:
  confidence [options] <directory_one> <directory_two>
  confidence (-h | --help)
  confidence --version

Options:
  -i, --ignore-errors  Ignore read errors so you can skip
                       files you don't have permission to read.
                       Useful for examining everything on a drive
                       that your non-root user can see.
";


fn actual_runtime(args: docopt::ArgvMap) -> i32 {
    println!("{:?}", args);

    if args.get_bool("--version") {
        println!("{}", VERSION);
        return 0;
    }

    let ignore_io_errors = args.get_bool("--ignore-errors");

    let filename_l = args.get_str("<directory_one>");
    let filename_r = args.get_str("<directory_two>");

    let walker = if ignore_io_errors {
        WalkDir::new(filename_l).into_iter().filter_map(|e| e.ok())
    }
    else {
        WalkDir::new(filename_l)
    };

    for entry in walker {
        match entry {
            Ok(entry) => {
                println!("{:?}", entry.path().display());
            },
            Err(error) => {
                println!("{}", error.to_string());
                return 1;
            }
        }
    }

    0
}


fn main() {
  let args = Docopt::new(USAGE)
                    .and_then(|dopt| dopt.parse())
                    .unwrap_or_else(|e| e.exit());

  std::process::exit(actual_runtime(args));
}

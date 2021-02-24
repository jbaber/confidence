use docopt::Docopt;
use walkdir::WalkDir;

const VERSION: &str = "0.1.0";

const USAGE: &str = "Usage:
  confidence [options] <directory_one> <directory_two>
  confidence (-h | --help)
  confidence --version
";


fn actual_runtime(args: docopt::ArgvMap) -> i32 {
    println!("{:?}", args);

    if args.get_bool("--version") {
        println!("{}", VERSION);
        return 0;
    }

    let filename_l = args.get_str("<directory_one>");
    let filename_r = args.get_str("<directory_two>");

    for entry in WalkDir::new(filename_l) {
        match entry {
            Ok(entry) => {
                println!("{:?}", entry.path().display());
            },
            Err(error) => {
                println!("{:?}", error);
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

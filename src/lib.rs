use walkdir::WalkDir;

const VERSION: &str = "0.1.0";

pub fn actual_runtime(args: docopt::ArgvMap) -> i32 {
    println!("{:?}", args);

    if args.get_bool("--version") {
        println!("{}", VERSION);
        return 0;
    }

    let ignore_io_errors = args.get_bool("--ignore-errors");

    let filename_l = args.get_str("<directory_one>");
    let filename_r = args.get_str("<directory_two>");

    for entry in WalkDir::new(filename_l) {
        match entry {
            Ok(entry) => {
                println!("{:?}", entry.path().display());
            },
            Err(error) => {
                if ignore_io_errors {
                    continue;
                }
                else {
                    println!("{}", error.to_string());
                    return 1;
                }
            }
        }
    }

    0
}



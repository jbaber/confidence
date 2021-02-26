use walkdir::WalkDir;

const VERSION: &str = "0.1.0";


fn runtime_with_regular_args(version_flag: bool, ignore_io_errors_flag: bool,
        filename_l: &str, filename_r: &str) -> i32 {
    if version_flag {
        println!("{}", VERSION);
        return 0;
    }

    for entry in WalkDir::new(filename_l) {
        match entry {
            Ok(entry) => {
                println!("{:?}", entry.path().display());
            },
            Err(error) => {
                if ignore_io_errors_flag {
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



pub fn actual_runtime(args: docopt::ArgvMap) -> i32 {
    println!("{:?}", args);

    runtime_with_regular_args(args.get_bool("--version"),
            args.get_bool("--ignore-errors"),
            args.get_str("<directory_one>"),
            args.get_str("<directory_two>"))
}

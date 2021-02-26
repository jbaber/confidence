use walkdir::WalkDir;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;

const VERSION: &str = "0.1.0";


pub fn runtime_with_regular_args(version_flag: bool, ignore_io_errors_flag: bool,
        filename_l: &str, filename_r: &str,
        mut writable: impl Write) -> Result<i32, Error> {
    if version_flag {
        writeln!(writable, "{}", VERSION)?;
        return Ok(0);
    }

    for entry in WalkDir::new(filename_l) {
        match entry {
            Ok(entry) => {
                writeln!(writable, "{}", entry.path().display())?;
            },
            Err(error) => {
                if ignore_io_errors_flag {
                    continue;
                }
                else {
                    writeln!(writable, "{}", error.to_string())?;
                    return Err(Error::new(ErrorKind::Other, error));
                }
            }
        }
    }

    Ok(0)
}



pub fn actual_runtime(args: docopt::ArgvMap) -> i32 {
    println!("{:?}", args);

    match runtime_with_regular_args(args.get_bool("--version"),
            args.get_bool("--ignore-errors"),
            args.get_str("<directory_one>"),
            args.get_str("<directory_two>"),
            std::io::stdout()) {
        Ok(retval) => {
            retval
        },
        Err(error) => {
            println!("{:?}", error);
            1
        }
    }
}

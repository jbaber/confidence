use walkdir::WalkDir;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use clap::ArgMatches;


pub fn runtime_with_regular_args(ignore_perm_errors_flag: bool,
        num_bytes: u64, filename_l: &str, filename_r: Option<&str>,
        mut writable: impl Write) -> Result<i32, Error> {
    for entry in WalkDir::new(filename_l) {
        match entry {
            Ok(entry) => {
                writeln!(writable, "{}", entry.path().display())?;
            },

            /* A lot of dancing around to return a regular io::Error instead of walkdir::Error
             * Maybe this can be avoided. */
            Err(error) => {
                match error.io_error() {
                    Some(io_error) => {
                        let kind = io_error.kind();
                        match kind {
                            ErrorKind::NotFound => {
                                return Err(Error::new(kind, error));
                            },
                            ErrorKind::PermissionDenied => {
                                if ignore_perm_errors_flag {
                                    continue;
                                }
                                return Err(Error::new(kind, error));
                            },
                            _ => {
                                return Err(Error::new(kind, error));
                            }
                        }
                    },

                    /* Doesn't correspond to IO error, e.g. cycle following
                     * symbolic links */
                    None => {
                        return Err(Error::new(ErrorKind::Other, error));
                    }
                }
            }
        }
    }

    Ok(0)
}



pub fn actual_runtime(matches: ArgMatches) -> i32 {

    /* Parse and validate arguments */
    let ignore_perm_errors_flag = matches.is_present("ignore-permission-errors");
    let size_arg = matches.value_of("size").unwrap();
    let mut num_bytes: u64;
    if let Ok(number) = size_arg.parse::<u64>() {
        num_bytes = number;
    }
    else {
        println!("Couldn't interpret '{}' as a number of bytes.", size_arg);
        return 1;
    }
    let filename_l = matches.value_of("directory_one").unwrap();
    let filename_r = matches.value_of("directory_two");

    /* Run them through the meat of the program */
    match runtime_with_regular_args(ignore_perm_errors_flag, num_bytes,
            filename_l, filename_r, std::io::stdout()) {
        Ok(retval) => {
            retval
        },
        Err(error) => {
            let outer_error_string = error.to_string();
            // TODO Any shorthand for this nested chain of matches?  if let?, unwrap, expect, ? 
            match error.kind() {
                ErrorKind::NotFound => {
                    match filename_r {
                        Some(filename) => println!("File named \"{}\" and/or \"{}\" couldn't be found.", filename_l, filename),
                        None => println!("File named \"{}\" couldn't be found.", filename_l),
                    }
                },
                ErrorKind::PermissionDenied => {
                    match error.into_inner() {
                        Some(inner_error) => {
                            match inner_error.downcast::<walkdir::Error>() {
                                Ok(inner_inner_error) => {
                                    match inner_inner_error.path() {
                                        Some(path) => {
                                            println!("Permission denied on '{}'.\nIf you want to move past such errors, use '--ignore-permission-errors'", path.display());
                                        },
                                        _ => {
                                            println!("Unexpected error: \"{}\"", outer_error_string);
                                        }
                                    }
                                },
                                _ => {
                                    println!("Unexpected error: \"{}\"", outer_error_string);
                                }
                            }
                        },
                        _ => {
                            println!("Unexpected error: \"{}\"", outer_error_string);
                        }
                    }

                },
                _ => {
                    println!("Unexpected error: \"{}\"", outer_error_string);
                }
            }
            1
        }
    }
}

use clap::ArgMatches;
use same_file::Handle;
use std::fs;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;


// TODO
// TODO Handle sizes bigger than u64::MAX
/// Returns number of bytes hashed
/// Writes out hash for later comparison
// pub fn hash_path(path: &Path, filename_l: &str, writable: &mut impl Write,
//         num_vs: u64) -> Result<u64, Error>{
//     writeln!(writable, "Output hash of {}", path.display())?;
//     Ok(0)
// }


// TODO Handle sizes bigger than u64::MAX
/// Returns number of bytes compared
/// Expect path to be a regular file,`filename_l` to be the directory
/// it's in.  `filename_r` to be a directory that should have a copy
/// of it.
pub fn compare_paths(path: &Path, filename_l: &str, filename_r: &str,
        writable: &mut impl Write, num_vs: u64) -> Result<u64, Error> {

    /* Don't care about directories or symlinks */
    if !path.is_file() {
        return Ok(0);
    }

    match path.strip_prefix(filename_l) {
        Ok(main_part) => {
            let path_l = Path::new(filename_l).join(main_part);
            let path_r = Path::new(filename_r).join(main_part);

            if num_vs > 1 {
                writeln!(writable, "Compare {} to {}", path_l.display(),
                        path_r.display())?;
            }

            if path_l != path {
                let error_s = "'".to_owned() + filename_l +
                        "' doesn't contain '" + path.to_str().unwrap() +
                        "'.  Don't know what to do here.";
                writeln!(writable, "{}", error_s)?;
                return Err(Error::new(ErrorKind::Other, error_s));
            }

            if !path_r.is_file() {
                let error_s = "'".to_owned() + path_r.to_str().unwrap() +
                        "' isn't a regular file, but '" +
                        path_l.to_str().unwrap() + "' is.";
                writeln!(writable, "{}", error_s)?;
                return Err(Error::new(ErrorKind::Other, error_s));
            }

            /* Finally, path_l and path_r are files to compare. */
            let metadata_l = fs::metadata(&path_l)?;
            let metadata_r = fs::metadata(&path_r)?;

            /* Be happy if they're literally the same file. */
            let num_bytes_l = metadata_l.len();
            if Handle::from_path(&path_l)? == Handle::from_path(&path_r)? {
                return Ok(num_bytes_l);
            }

            // TODO Don't just panic here.
            let path_l_s = path_l.to_str().unwrap();
            let path_r_s = path_r.to_str().unwrap();

            /* Be unhappy if they're different sizes */
            if num_bytes_l != metadata_r.len() {
                let error_s = "'".to_owned() + path_l.to_str().unwrap() +
                        "' and '" + path_r_s + "' aren't the same size.";
                writeln!(writable, "{}", error_s)?;
                return Err(Error::new(ErrorKind::Other, error_s));
            }

            /* Finally, compare their contents */
            let mut file_l = File::open(&path_l)?;
            let mut file_r = File::open(&path_r)?;
            let mut buffer_l = [0; 32];
            let mut buffer_r = [0; 32];

            let mut num_bytes_compared = 0;
            let mut done = false;
            while !done {
                let num_bytes_read_l = file_l.read(&mut buffer_l[..])?;
                let num_bytes_read_r = file_r.read(&mut buffer_r[..])?;

                if num_bytes_read_l != num_bytes_read_r {
                    // TODO Get rid of unwraps that allow panicking.
                    let error_s =
                            "Couldn't read the same number of ".to_owned() +
                            "bytes from '" + path_l_s +
                            "' and '" + path_r_s + "'";
                    writeln!(writable, "{}", error_s)?;
                    return Err(Error::new(ErrorKind::Other, error_s));
                }

                if buffer_l != buffer_r {
                    let error_s = "'".to_owned() + path_l_s +
                            "' and '" + path_r_s +
                            "' aren't equal.";
                    writeln!(writable, "{}", error_s)?;
                    return Err(Error::new(ErrorKind::Other, error_s));
                }

                /* At this point, we've actually compared bytes */
                num_bytes_compared += num_bytes_l;

                if num_bytes_l < 32 {
                    done = true;
                }
            }

            if num_vs > 1 {
                writeln!(writable, "Successfully compared {} bytes",
                        num_bytes_compared)?;
            }
            Ok(num_bytes_compared)
        },

        /* filename_l doesn't contain path*/
        Err(error) => {
            return Err(Error::new(ErrorKind::Other, error));
        }
    }
}


pub fn runtime_with_regular_args(ignore_perm_errors_flag: bool,
        num_bytes: u64, filename_l: &str, filename_r: Option<&str>,
        mut writable: impl Write, num_vs: u64) -> Result<i32, Error> {
    let mut num_bytes_compared = 0;
    for entry in WalkDir::new(filename_l) {
        match entry {
            Ok(entry) => {
                match filename_r {
                    Some(filename_r) => {
                        num_bytes_compared += compare_paths(entry.path(),
                                filename_l, filename_r, &mut writable,
                                num_vs)?;
                    },
                    None =>{
                        // hash_path(entry.path(), filename_l, &mut writable,
                        //     num_vs)?;
                    }
                }
            },

            /* A lot of dancing around to return a regular io::Error instead of
             * walkdir::Error. Maybe this can be avoided. */
            Err(error) => {
                match error.io_error() {
                    Some(io_error) => {
                        let kind = io_error.kind();
                        match kind {
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

    writeln!(writable, "{} of {} bytes agree.  ({}% confidence)",
            num_bytes_compared, num_bytes,
            (num_bytes_compared as f32 / num_bytes as f32) * 100.0)?;

    Ok(0)
}



pub fn actual_runtime(matches: ArgMatches) -> i32 {

    /* Parse and validate arguments */
    let ignore_perm_errors_flag = matches.is_present("ignore-permission-errors");
    let size_arg = matches.value_of("size").unwrap();
    let num_bytes: u64;
    if let Ok(number) = size_arg.parse::<u64>() {
        num_bytes = number;
    }
    else {
        println!("Couldn't interpret '{}' as a number of bytes.", size_arg);
        return 1;
    }
    let filename_l = matches.value_of("directory_one").unwrap();
    let filename_r = matches.value_of("directory_two");
    let num_vs = matches.occurrences_of("verbosity");

    /* Run them through the meat of the program */
    match runtime_with_regular_args(ignore_perm_errors_flag, num_bytes,
            filename_l, filename_r, std::io::stdout(), num_vs) {
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

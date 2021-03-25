use base64;
use clap::ArgMatches;
use same_file::Handle;
use sha1;
use std::cmp;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::Add;
use std::ops::AddAssign;
use std::path::Path;
use walkdir::WalkDir;
use indicatif::ProgressBar;


pub struct BytesComparison {
    disagreement: usize,
    agreement: usize,
}


impl Add for BytesComparison {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            disagreement: self.disagreement + other.disagreement,
            agreement: self.agreement + other.agreement,
        }
    }
}


impl AddAssign for BytesComparison {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            disagreement: self.disagreement + other.disagreement,
            agreement: self.agreement + other.agreement,
        };
    }
}


pub fn size_from_path(path: &Path) -> Result<usize, Error> {
    if !path.is_file() {
        if let Some(path_s) = path.to_str() {
            Err(Error::new(ErrorKind::Other,
                    path_s.to_owned() + " is not a regular file."))
        }
        else {
            Err(Error::new(ErrorKind::Other, "Empty path"))
        }
    }
    else {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len() as usize)
    }
}


/// Returns the hash string and the number of bytes hashed
pub fn hash_of_path(path: &Path) -> Result<(String, usize), Error> {
    if !path.is_file() {
        match path.to_str() {
            Some(path_s) => {
                return Err(Error::new(ErrorKind::Other,
                        path_s.to_owned() + " is not a regular file."));
            },
            None => {
                return Err(Error::new(ErrorKind::Other, "Empty path"));
            }
        }
    }

    let mut cur_hash = sha1::Sha1::new();
    let mut file = File::open(&path)?;
    let mut buffer: [u8; 32] = [0; 32];
    let mut num_bytes_hashed: usize = 0;
    let mut done = false;
    while !done {
        let num_bytes_read = file.read(&mut buffer[..])?;

        let bytes_to_hash = &buffer[..num_bytes_read];

        cur_hash.update(bytes_to_hash);
        num_bytes_hashed += num_bytes_read;
        if num_bytes_read < 32 {
            done = true;
        }
    }

    Ok((cur_hash.digest().to_string(), num_bytes_hashed))
}


// TODO Do hashes other than sha1
/// Returns number of bytes hashed
/// Writes out hash for later comparison
pub fn hash_path(path: &Path, filename_l: &str,
        writable: &mut impl Write, num_vs: u8,
        progress_bar: &Option<ProgressBar>) -> Result<usize, Error> {
    if num_vs > 1 {
        eprintln!("Output hash of {}", path.display());
    }

    if !path.is_file() {
        return Ok(0);
    }

    let possibly_error = hash_of_path(path);

    match possibly_error {
        Ok((cur_hash, num_bytes_hashed)) => {
            output_progress(num_bytes_hashed as u64, &progress_bar);
            if num_vs > 1 {
                eprintln!("Successfully hashed {} bytes",
                        num_bytes_hashed);
            }

            // TODO Maybe use serde or something so the path isn't just
            // whatever'd be displayed?  (Weirdo unicode characters, etc.)
            match path.strip_prefix(filename_l) {
                Ok(main_part) => {

                    /* Get the path as a string, then base64 it so it has no spaces
                     * When it's read back into a Path, it'll have to be converted
                     * from a vector of u8's (this is unix specific!) to an OsStr */
                    if let Some(path_s) = main_part.to_str() {
                        writeln!(writable, "sha1: {} {} {}", cur_hash,
                                base64::encode(path_s), num_bytes_hashed)?;
                    }
                    else {
                        return Err(Error::new(ErrorKind::Other,
                                "Could not cast path to a string"));
                    }
                },
                Err(error) => {
                    return Err(Error::new(ErrorKind::Other, error));
                }

            }
            Ok(num_bytes_hashed)
        },
        Err(error) => Err(error)
    }
}


// TODO Handle sizes bigger than u64::MAX
/// Expect path to be a regular file,`filename_l` to be the directory
/// it's in.  `filename_r` to be a directory that should have a copy
/// of it.
pub fn compare_paths(path: &Path, filename_l: &str, filename_r: &str,
        writable: &mut impl Write, num_vs: u8) -> Result<BytesComparison, Error> {

    /* Don't care about directories or symlinks */
    if !path.is_file() {
        return Ok(BytesComparison{disagreement: 0, agreement:0});
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
                return Ok(BytesComparison{disagreement: size_from_path(&path_l)?, agreement: 0});
            }

            /* Finally, path_l and path_r are files to compare. */
            let metadata_l = fs::metadata(&path_l)?;
            let metadata_r = fs::metadata(&path_r)?;

            /* Be happy if they're literally the same file. */
            let num_bytes_l = metadata_l.len() as usize;
            if Handle::from_path(&path_l)? == Handle::from_path(&path_r)? {
                return Ok(BytesComparison{agreement: num_bytes_l, disagreement: 0});
            }

            // TODO Don't just panic here.
            let path_l_s = path_l.to_str().unwrap();
            let path_r_s = path_r.to_str().unwrap();

            /* Be unhappy if they're different sizes */
            let num_bytes_r = metadata_r.len() as usize;
            let max_bytes_compared = cmp::max(num_bytes_l, num_bytes_r);
            if num_bytes_l != num_bytes_r {
                let error_s = "'".to_owned() + path_l.to_str().unwrap() +
                        "' and '" + path_r_s + "' aren't the same size.";
                writeln!(writable, "{}", error_s)?;
                return Ok(BytesComparison{disagreement: max_bytes_compared, agreement: 0});
            }

            /* Finally, compare their contents */
            let mut file_l = File::open(&path_l)?;
            let mut file_r = File::open(&path_r)?;
            let mut buffer_l = [0; 32];
            let mut buffer_r = [0; 32];

            let mut num_bytes_examined = 0;
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
                    return Ok(BytesComparison{disagreement: max_bytes_compared, agreement: 0});
                }

                if buffer_l != buffer_r {
                    let error_s = "'".to_owned() + path_l_s +
                            "' and '" + path_r_s +
                            "' aren't equal.";
                    writeln!(writable, "{}", error_s)?;
                    return Ok(BytesComparison{disagreement: max_bytes_compared, agreement: 0});
                }

                /* At this point, we've actually compared bytes */
                num_bytes_examined += num_bytes_l;

                if num_bytes_l < 32 {
                    done = true;
                }
            }

            if num_vs > 1 {
                writeln!(writable, "Successfully compared {} bytes",
                        num_bytes_examined)?;
            }
            Ok(BytesComparison{agreement: num_bytes_examined, disagreement: 0})
        },

        /* filename_l doesn't contain path*/
        Err(error) => {
            return Err(Error::new(ErrorKind::Other, error));
        }
    }
}



/// Returns last line of `open_file` as a String
/// Ignores the last `\n` that's usually there in unix.
pub fn last_line_of(mut open_file: &std::fs::File) -> Result<String, Error> {

    /* Seek to last line.  Iterate backwards from final byte to find
     * the penultimate \n, then read forward to get last line.
     * TODO Care about crlf or whatever. */
    let original_position = open_file.seek(SeekFrom::Current(0))?;
    let metadata = open_file.metadata()?;
    let mut last_line_byte_num: u64 = 0;
    let hashes_file_num_bytes = metadata.len();
    let mut cur_byte = [0];

    /* Last byte usually *is* a newline in unix, so start at penultimate
     * byte */
    for byte_num in (0..hashes_file_num_bytes - 1).rev() {
        open_file.seek(SeekFrom::Start(byte_num))?;
        let num_bytes_read = open_file.read(&mut cur_byte)?;
        if num_bytes_read != 1 {
            return Err(Error::new(ErrorKind::Other,
                    "Couldn't read from file"));
        }
        if cur_byte[0] == 0x0a {
            last_line_byte_num = byte_num;
            break;
        }
    }

    /* If we haven't errored out, we're on the last line */
    open_file.seek(SeekFrom::Start(last_line_byte_num + 1))?;
    let mut last_line = String::new();
    let _ = open_file.read_to_string(&mut last_line);
    open_file.seek(SeekFrom::Start(original_position))?;

    Ok(last_line)
}


pub fn bytes_from_last_line(last_line: &str) -> Result<usize, Error> {
    let pieces = last_line.split_whitespace().collect::<Vec<_>>();
    if pieces.len() != 3 || pieces[1] != "bytes" || pieces[2] != "hashed" {
        return Err(Error::new(ErrorKind::Other,
                "Corrupt file (doesn't end with 'XXX bytes hashed')"));
    }

    let num_bytes_hashed = pieces[0].parse::<usize>();
    if num_bytes_hashed.is_err() {
        let err_s = "Can't interpret ".to_owned() + pieces[0] + " as an integer.";
        return Err(Error::new(ErrorKind::Other, err_s));
    }

    Ok(num_bytes_hashed.unwrap())
}


/// Un-base64 the path to a regular string.  Unix specific.
pub fn path_string_from_b64(b64: &str) -> Result<String, Error> {
    let path_vec_u8 = base64::decode(b64);
    let path_s: String;
    match path_vec_u8 {
        Ok(u8s) => {
            let possibly_path_s = std::str::from_utf8(&u8s);
            match possibly_path_s {
                Ok(s) => {
                    path_s = s.to_owned();
                }
                Err(_) => {
                    let err_s = "Couldn't convert bytes from unbased64'd {} t\
                            o a path".to_owned() + b64;
                    return Err(Error::new(ErrorKind::Other, err_s));
                }
            }
        }
        Err(_) => {
            let err_s = "Couldn't unbase64 ".to_owned() + b64 + " to a path";
            return Err(Error::new(ErrorKind::Other, err_s));
        }
    }

    Ok(path_s)
}


pub fn compare_hashes(hashes_filename: &str, directory: &str, num_vs: u8,
        num_bytes: Option<usize>, mut writable: impl Write) ->
        Result<BytesComparison, Error> {
    if num_vs > 1 {
        writeln!(writable, "Reading {}", hashes_filename)?;
    }
    let hashes_path = Path::new(hashes_filename);
    let hashes_file = File::open(&hashes_path)?;
    let num_bytes_hashed = bytes_from_last_line(&last_line_of(&hashes_file)?)?;
    if num_vs > 0 {
        writeln!(writable, "Num bytes previously hashed: {}", num_bytes_hashed)?;
    }

    /* Iterate line by line (except the final line) */
    let reader = BufReader::new(hashes_file);
    let mut to_return = BytesComparison{agreement: 0, disagreement: 0};
    for line in reader.lines() {
        let line = line.unwrap();
        let pieces = line.split_whitespace().collect::<Vec<_>>();

        /* Quit at last line */
        if pieces.len() == 3 && pieces[1] == "bytes" && pieces[2] == "hashed" {
            break;
        }

        if pieces.len() != 4 {
            return Err(Error::new(ErrorKind::Other,
                    "Corrupt file: found a line without 4 components"));
        }

        let sha1 = pieces[1];

        let num_bytes_hashed = pieces[3].parse::<usize>();
        if num_bytes_hashed.is_err() {
            let err_s = "Can't interpret ".to_owned() + pieces[3] +
                    " as an integer.";
            return Err(Error::new(ErrorKind::Other, err_s));
        }
        let num_bytes_hashed = num_bytes_hashed.unwrap();

        let unbased = path_string_from_b64(pieces[2])?;
        let path = Path::new(directory).join(Path::new(&unbased));

        if num_vs > 1 {
            writeln!(writable, "Examining {}", path.display())?;
        }

        /* Don't bother to hash if filesizes don't match */
        let cur_size = size_from_path(&path)?;
        let max_bytes_compared = cmp::max(cur_size, num_bytes_hashed);
        if cur_size != num_bytes_hashed {
            to_return += BytesComparison{disagreement: max_bytes_compared, agreement: 0};
            continue;
        }

        match hash_of_path(&path) {
            Ok(hash_and_size) => {
                let hash_s = hash_and_size.0;
                let num_bytes_hashed = hash_and_size.1;
                if sha1 == hash_s {
                    to_return += BytesComparison{disagreement: 0, agreement: num_bytes_hashed};
                    continue;
                }
                else {
                    to_return += BytesComparison{disagreement: max_bytes_compared, agreement: 0};
                    continue;
                }
            },
            Err(_) => {
                writeln!(writable, "Couldn't hash {}", path.display())?;
                to_return += BytesComparison{disagreement: max_bytes_compared, agreement: 0};
                continue;
            }
        }
    }

    match num_bytes {
        None => {
            writeln!(writable, "Agreed on {} bytes.", to_return.agreement)?;
            if to_return.disagreement > 0 {
                writeln!(writable, "Disagreed on {} bytes.",
                        to_return.disagreement)?;
            }
        },
        Some(num_bytes) => {
            writeln!(writable,
                    "Agreed on {}/{} bytes ({}% confidence)",
                    to_return.agreement, num_bytes,
                    ((to_return.agreement as f32 / num_bytes as f32) * 100.0))?;
            if to_return.disagreement > 0 {
                writeln!(writable,
                    "Disagreed on {}/{} bytes ({}% worry)",
                    to_return.disagreement, num_bytes,
                    ((to_return.disagreement as f32 / num_bytes as f32) * 100.0))?;
            }
        },
    }

    Ok(to_return)
}


/// Print a progress bar to stderr if `progress` is true and `num_bytes` is_some
pub fn output_progress(numerator: u64, progress_bar: &Option<ProgressBar>) {
    if let Some(ref bar) = progress_bar {
      bar.inc(numerator as u64);
    }
}


/// Return number of bytes 
pub fn runtime_with_regular_args(ignore_perm_errors_flag: bool,
        num_bytes: Option<usize>, filename_l: &str, filename_r: Option<&str>,
        hashes_filename: Option<&str>, mut writable: impl Write, num_vs: u8,
        progress: bool) ->
        Result<i32, Error> {
    let progress_bar = if progress && num_bytes.is_some() {
        Some(ProgressBar::new(num_bytes.unwrap() as u64))
    }
    else {
        None
    };
    let comparing_paths = filename_r.is_some();
    let comparing_hashes = hashes_filename.is_some();

    if comparing_hashes {
        match compare_hashes(hashes_filename.unwrap(), filename_l, num_vs, num_bytes, writable) {
            Err(error) => {
                return Err(Error::new(ErrorKind::Other, error));
            },
            Ok(bytes_comparison) => {
                if bytes_comparison.disagreement > 0 {
                    return Ok(1);
                }
                else {
                    return Ok(0);
                }
            },
        }
    }

    /* Otherwise, walk the tree now */
    let mut bytes_compared = BytesComparison{disagreement: 0, agreement: 0};
    let mut bytes_examined: usize = 0;
    for entry in WalkDir::new(filename_l) {
        match entry {
            Ok(entry) => {
                if comparing_paths {
                    bytes_compared += compare_paths(entry.path(),
                            filename_l, filename_r.unwrap(), &mut writable,
                            num_vs)?;
                }

                /* This is the generated hashes case */
                else {
                    bytes_examined += hash_path(entry.path(), filename_l,
                            &mut writable, num_vs, &progress_bar)?;
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

    // TODO This should only be written out when comparing to another
    // directory or a list of hashes
    if comparing_paths {
        match num_bytes {
            Some(num_bytes) => {
                writeln!(writable, "{} of {} bytes agree.  ({}% confidence)",
                        bytes_compared.agreement, num_bytes,
                        (bytes_compared.agreement as f32 / num_bytes as f32) * 100.0)?;
                if bytes_compared.disagreement > 0 {
                    writeln!(writable, "{} of {} bytes disagree.  ({}% worry)",
                            bytes_compared.disagreement, num_bytes,
                            (bytes_compared.disagreement as f32 / num_bytes as f32) * 100.0)?;
                }
            },
            None => {
                writeln!(writable, "{} bytes agree.",
                        bytes_compared.agreement)?;
                if bytes_compared.disagreement > 0 {
                    writeln!(writable, "{} bytes disagree.",
                            bytes_compared.agreement)?;
                }
            }
        }
    }
    else {
        writeln!(writable, "{} bytes hashed", bytes_examined)?;
        return Ok(0);
    }

    if bytes_compared.disagreement > 0 {
        Ok(1)
    }
    else {
        Ok(0)
    }
}



pub fn actual_runtime(matches: ArgMatches) -> i32 {

    /* Parse and validate arguments */
    let ignore_perm_errors_flag =
            matches.is_present("ignore-permission-errors");
    let progress = matches.is_present("progress");
    let num_bytes: Option<usize>;
    match matches.value_of("size") {
        Some(size_arg) => {
            if let Ok(number) = size_arg.parse::<usize>() {
                num_bytes = Some(number);
            }
            else {
                println!("Couldn't interpret '{}' as a number of bytes.",
                        size_arg);
                return 1;
            }
        },
        None => {
            num_bytes = None;
        }
    }

    let filename_l = matches.value_of("directory_one").unwrap();
    let filename_r = matches.value_of("directory_two");
    let num_vs = matches.occurrences_of("verbosity") as u8;
    let input_filename = matches.value_of("input");
    let output_file = match matches.value_of("output") {
        Some(filename) => {
            match File::create(filename) {
                Ok(file) => {
                    Box::new(file) as Box<dyn Write>
                },
                Err(_error) => {
                    println!("Couldn't open '{}' for writing.", filename);
                    return 2;
                }
            }
        },
        None => Box::new(std::io::stdout()) as Box<dyn Write>,
    };

    /* Run them through the meat of the program */
    match runtime_with_regular_args(ignore_perm_errors_flag, num_bytes,
            filename_l, filename_r, input_filename, output_file, num_vs,
            progress) {
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
                                            println!("Permission denied on '{}' -- aborting.\nIf you want to move past such errors, use '--ignore-permission-errors'", path.display());
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

use docopt::Docopt;

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


fn main() {
  let args = Docopt::new(USAGE)
                    .and_then(|dopt| dopt.parse())
                    .unwrap_or_else(|e| e.exit());

  std::process::exit(confidence::actual_runtime(args));
}

- use async/await to for all the functions that do things like count bytes in a file, get the sha, etc.
- Add a command that finds the number of bytes in files only
- Currently specific to unix where filenames can be seen as a vector of u8's.  Generalize
- Use serde to perform serialization of path to base64 (just trying to remove spaces).
- Make verbose output go to stderr (since stdout should become an infile later)

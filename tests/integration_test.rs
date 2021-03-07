use confidence::runtime_with_regular_args;

#[test]
fn test_dir_0() {
    let mut stdout = Vec::new();
    let result = runtime_with_regular_args(true, 19, "tests/test_dir_0",
            Some("tests/test_dir_1"), &mut stdout, 3);
    assert_eq!(result.unwrap(), 0);

    // TODO Remove sentinel files before test.  They're only there because
    // empty directories can't be versioned.
    assert_eq!(std::str::from_utf8(&stdout).unwrap(), r###"Compare tests/test_dir_0/a/b/c/r to tests/test_dir_1/a/b/c/r
Successfully compared 2 bytes
Compare tests/test_dir_0/a/b/c/s to tests/test_dir_1/a/b/c/s
Successfully compared 2 bytes
Compare tests/test_dir_0/a/b/c/d/e/f/g/h/i/j/sentinel to tests/test_dir_1/a/b/c/d/e/f/g/h/i/j/sentinel
Successfully compared 0 bytes
Compare tests/test_dir_0/k/t to tests/test_dir_1/k/t
Successfully compared 2 bytes
Compare tests/test_dir_0/k/l/m/u to tests/test_dir_1/k/l/m/u
Successfully compared 2 bytes
Compare tests/test_dir_0/k/l/m/w to tests/test_dir_1/k/l/m/w
Successfully compared 2 bytes
Compare tests/test_dir_0/k/l/m/v to tests/test_dir_1/k/l/m/v
Successfully compared 2 bytes
Compare tests/test_dir_0/k/l/m/n/sentinel to tests/test_dir_1/k/l/m/n/sentinel
Successfully compared 0 bytes
Compare tests/test_dir_0/y to tests/test_dir_1/y
Successfully compared 2 bytes
Compare tests/test_dir_0/o/x to tests/test_dir_1/o/x
Successfully compared 3 bytes
Compare tests/test_dir_0/o/p/q/z to tests/test_dir_1/o/p/q/z
Successfully compared 2 bytes
19 of 19 bytes agree.  (100% confidence)
"###);
}

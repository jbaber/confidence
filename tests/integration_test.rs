use confidence::runtime_with_regular_args;

#[test]
fn version_flag() {
    let mut stdout = Vec::new();
    let result = runtime_with_regular_args(true, true, "tests/test_dir_0",
            "tests/test_dir_1", &mut stdout);
    assert_eq!(result.unwrap(), 0);
    assert_eq!(stdout, b"0.1.0\n");
}

#[test]
fn test_dir_0() {
    let mut stdout = Vec::new();
    let result = runtime_with_regular_args(false, false, "tests/test_dir_0",
            "tests/test_dir_1", &mut stdout);
    assert_eq!(result.unwrap(), 0);
    assert_eq!(stdout, r###"tests/test_dir_0
tests/test_dir_0/a
tests/test_dir_0/a/b
tests/test_dir_0/a/b/c
tests/test_dir_0/a/b/c/r
tests/test_dir_0/a/b/c/s
tests/test_dir_0/a/b/c/d
tests/test_dir_0/a/b/c/d/e
tests/test_dir_0/a/b/c/d/e/f
tests/test_dir_0/a/b/c/d/e/f/g
tests/test_dir_0/a/b/c/d/e/f/g/h
tests/test_dir_0/a/b/c/d/e/f/g/h/i
tests/test_dir_0/a/b/c/d/e/f/g/h/i/j
tests/test_dir_0/k
tests/test_dir_0/k/t
tests/test_dir_0/k/l
tests/test_dir_0/k/l/m
tests/test_dir_0/k/l/m/u
tests/test_dir_0/k/l/m/w
tests/test_dir_0/k/l/m/v
tests/test_dir_0/k/l/m/n
tests/test_dir_0/y
tests/test_dir_0/o
tests/test_dir_0/o/x
tests/test_dir_0/o/p
tests/test_dir_0/o/p/q
tests/test_dir_0/o/p/q/z
"###.as_bytes());
}

use confidence::runtime_with_regular_args;

#[test]
fn version_flag() {
    let mut stdout = Vec::new();
    let result = runtime_with_regular_args(true, true, "test_dir_0",
            "test_dir_1", &mut stdout);
    assert_eq!(result.unwrap(), 0);
    assert_eq!(stdout, b"0.1.0\n");
}

#[test]
fn test_dir_0() {
    let mut stdout = Vec::new();
    let result = runtime_with_regular_args(false, false, "test_dir_0",
            "test_dir_1", &mut stdout);
    assert_eq!(result.unwrap(), 0);
    assert_eq!(stdout, r###"test_dir_0
test_dir_0/a
test_dir_0/a/b
test_dir_0/a/b/c
test_dir_0/a/b/c/r
test_dir_0/a/b/c/s
test_dir_0/a/b/c/d
test_dir_0/a/b/c/d/e
test_dir_0/a/b/c/d/e/f
test_dir_0/a/b/c/d/e/f/g
test_dir_0/a/b/c/d/e/f/g/h
test_dir_0/a/b/c/d/e/f/g/h/i
test_dir_0/a/b/c/d/e/f/g/h/i/j
test_dir_0/k
test_dir_0/k/t
test_dir_0/k/l
test_dir_0/k/l/m
test_dir_0/k/l/m/u
test_dir_0/k/l/m/w
test_dir_0/k/l/m/v
test_dir_0/k/l/m/n
test_dir_0/y
test_dir_0/o
test_dir_0/o/x
test_dir_0/o/p
test_dir_0/o/p/q
test_dir_0/o/p/q/z
"###.as_bytes());
}

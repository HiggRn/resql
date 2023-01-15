use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::{Command, Stdio};

fn run(commands: Vec<String>, filename: &str) -> (Vec<String>, Vec<String>) {
    let mut child = Command::new("cargo")
        .arg("run")
        .arg(filename)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn child process");

    let mut stdin = child.stdin.take().expect("failed to get stdin");

    // If the child process fills its stdout buffer, it may end up
    // waiting until the parent reads the stdout, and not be able to
    // read stdin in the meantime, causing a deadlock.
    // Writing from another thread ensures that stdout is being read
    // at the same time, avoiding the problem.
    let handle = std::thread::spawn(move || {
        commands.iter().for_each(|cmd| {
            stdin
                .write_all(&[cmd.as_bytes(), b"\n"].concat())
                .expect("failed to write to stdin");
        });
    });

    // wait for output also attempts to read from the buffer for stdout && stderr which stops us from hanging
    let output = child.wait_with_output().expect("failed to read stdout");
    handle.join().unwrap();
    let out: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .to_string()
        .split("\n")
        .map(String::from)
        .collect();
    let err: Vec<String> = String::from_utf8_lossy(&output.stderr)
        .to_string()
        .split("\n")
        .map(String::from)
        .collect();

    (out, err)
}

fn ensure_clean_fs<P: AsRef<Path>>(test_filename: P) {
    std::fs::remove_file(test_filename)
        .or_else(|e| match e.kind() {
            ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })
        .expect("could not clean up database files before running tests");
}

fn clean_test(test_case: &str, test: fn(&str)) -> impl Fn() {
    let test_filename = format!("test_db_{}.db", test_case);
    let clean_test_wrapper = move || {
        ensure_clean_fs(&test_filename);
        test(&test_filename);
        ensure_clean_fs(&test_filename);
    };
    clean_test_wrapper
}

#[test]
fn test_insert_a_row() {
    let test_case = "insert_a_row";

    let test = |test_filename: &str| {
        let (out, _) = run(
            vec![
                "insert 1 user1 person1@example.com".into(),
                "select".into(),
                ".exit".into(),
            ],
            test_filename,
        );

        let mut contain = false;
        for s in out.iter() {
            if s.contains("1: user1 person1@example.com") {
                contain = true;
                break;
            }
        }
        assert!(contain);
    };

    clean_test(test_case, test)();
}

#[test]
fn test_parse_error() {
    let test_case = "parse_error";

    let test = |test_filename: &str| {
        let (_, err) = run(
            vec!["insert -32 user1 user1@example.com".into(), ".exit".into()],
            test_filename,
        );
        assert!(err[err.len() - 2].contains("[ERROR]can't parse '-32' to u32"));
    };

    clean_test(test_case, test)();
}

#[test]
fn test_syntax_error() {
    let test_case = "syntax_error";

    let test = |test_filename: &str| {
        let (_, err) = run(vec!["insert 1 user1".into(), ".exit".into()], test_filename);
        assert!(err[err.len() - 2].contains("[ERROR]syntax error"));
    };

    clean_test(test_case, test)();
}

#[test]
fn test_too_long() {
    let test_case = "too_long";

    let test = |test_filename: &str| {
        let (_, err) = run(
            vec![
                "insert 1 nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn abc".into(),
                ".exit".into(),
            ],
            test_filename,
        );
        assert!(err[err.len() - 2]
            .contains("[ERROR]'nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn' is too long for username"));
    };

    clean_test(test_case, test)();
}

#[test]
fn test_keep_data() {
    let test_case = "keep_data";

    let test = |test_filename: &str| {
        let _ = run(
            vec!["insert 1 user1 person1@example.com".into(), ".exit".into()],
            test_filename,
        );

        let (out, _) = run(vec!["select".into(), ".exit".into()], test_filename);

        let mut contain = false;
        for s in out.iter() {
            if s.contains("1: user1 person1@example.com") {
                contain = true;
                break;
            }
        }
        assert!(contain);
    };

    clean_test(test_case, test)();
}

#[test]
fn test_constants() {
    let test_case = "constants";

    let test = |test_filename: &str| {
        let (out, _) = run(vec![".constants".into(), ".exit".into()], test_filename);

        let mut contain = false;
        for s in out.iter() {
            if s.contains("COMMON_HEADER_SIZE") {
                contain = true;
                break;
            }
        }
        assert!(contain);
    };

    clean_test(test_case, test)();
}

#[test]
fn test_ordering() {
    let test_case = "ordering";

    let test = |test_filename: &str| {
        let (out, _) = run(
            vec![
                "insert 1 user1 person1@example.com".into(),
                "insert 2 user2 person2@example.com".into(),
                "insert 4 user4 person4@example.com".into(),
                "insert 5 user5 person5@example.com".into(),
                "insert 3 user3 person3@example.com".into(),
                "select".into(),
                ".exit".into(),
            ],
            test_filename,
        );

        for (num, line) in out[1..5].iter().enumerate() {
            let index = num + 2;
            assert_eq!(
                line,
                &format!("{index}: user{index} person{index}@example.com")
            );
        }
    };

    clean_test(test_case, test)();
}

#[test]
fn test_duplicate() {
    let test_case = "duplicate";

    let test = |test_filename: &str| {
        let (_, err) = run(
            vec![
                "insert 1 user1 person1@example.com".into(),
                "insert 2 user2 person2@example.com".into(),
                "insert 4 user4 person4@example.com".into(),
                "insert 5 user5 person5@example.com".into(),
                "insert 3 user3 person3@example.com".into(),
                "insert 2 user2 person2@example.com".into(),
                ".exit".into(),
            ],
            test_filename,
        );

        assert!(err[err.len() - 2].contains("duplicate key '2'"));
    };

    clean_test(test_case, test)();
}

#[test]
fn test_tree() {
    let test_case = "tree";

    let test = |test_filename: &str| {
        let mut cmds = Vec::new();
        for i in 1..15 {
            cmds.push(format!("insert {i} user{i} person{i}@example.com"));
        }
        cmds.push(".btree".into());
        cmds.push(".exit".into());
        let (out, _) = run(cmds, test_filename);
        let expected_out = [
            "- internal (size 1)",
            "  - leaf (size 7)",
            "    - key 1",
            "    - key 2",
            "    - key 3",
            "    - key 4",
            "    - key 5",
            "    - key 6",
            "    - key 7",
            "  - key 7",
            "  - leaf (size 7)",
            "    - key 8",
            "    - key 9",
            "    - key 10",
            "    - key 11",
            "    - key 12",
            "    - key 13",
            "    - key 14",
            "exitting...",
            "",
        ];
        for (i, s) in out.iter().enumerate() {
            let str = s.trim_start_matches(">> ").trim_end();
            assert_eq!(str, expected_out[i]);
        }
    };

    clean_test(test_case, test)();
}

#[test]
fn test_search_internal_node() {
    let test_case = "search_internal_node";

    let test = |test_filename: &str| {
        let mut cmds = Vec::new();
        for i in 1..16 {
            cmds.push(format!("insert {i} user{i} person{i}@example.com"));
        }
        cmds.push(".exit".into());
        let (_, err) = run(cmds, test_filename);
        for s in err.iter() {
            assert!(!s.contains("need to implement"));
        }
    };

    clean_test(test_case, test)();
}

#[test]
fn test_select() {
    let test_case = "select";

    let test = |test_filename: &str| {
        let mut cmds = Vec::new();
        for i in 1..16 {
            cmds.push(format!("insert {i} user{i} person{i}@example.com"));
        }
        cmds.push("select".into());
        cmds.push(".exit".into());
        let (out, _) = run(cmds, test_filename);
        let expected_out = [
            "1: user1 person1@example.com",
            "2: user2 person2@example.com",
            "3: user3 person3@example.com",
            "4: user4 person4@example.com",
            "5: user5 person5@example.com",
            "6: user6 person6@example.com",
            "7: user7 person7@example.com",
            "8: user8 person8@example.com",
            "9: user9 person9@example.com",
            "10: user10 person10@example.com",
            "11: user11 person11@example.com",
            "12: user12 person12@example.com",
            "13: user13 person13@example.com",
            "14: user14 person14@example.com",
            "15: user15 person15@example.com",
            "exitting...",
            "",
        ];
        for (i, s) in out.iter().enumerate() {
            let str = s.trim_start_matches(">> ").trim_end();
            assert_eq!(str, expected_out[i]);
        }
    };

    clean_test(test_case, test)();
}

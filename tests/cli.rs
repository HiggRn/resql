use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;

fn run(commands: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut child = Command::new("cargo")
        .arg("run")
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
    let handle = thread::spawn(move || {
        commands.iter().for_each(|cmd| {
            stdin.write_all(&[cmd.as_bytes(), b"\n"].concat())
                .expect("failed to write to stdin");
        });
    });

    // wait for output also attempts to read from the buffer for stdout && stderr which stops us from hanging
    let output = child.wait_with_output().expect("failed to read stdout");
    handle.join().unwrap();
    let out: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .to_string()
        .split("\n").map(String::from).collect();
    let err: Vec<String> = String::from_utf8_lossy(&output.stderr)
        .to_string()
        .split("\n").map(String::from).collect();

    (out, err)
}

#[test]
fn db_insert_a_row() {    
    let (out, _) = run(vec![
        "insert 1 user1 person1@example.com".into(),
        "select".into(),
        ".exit".into(),
    ]);

    let mut contain = false;
    for s in out.iter() {
        if s.contains("1: user1 person1@example.com") {
            contain = true;
        }
    }
    assert!(contain);
}

#[test]
fn db_parse_error() {
    let (_, err) = run(vec![
        "insert -32 user1 user1@example.com".into(),
        ".exit".into()
    ]);
    assert!(err[err.len() - 2].contains("[ERROR]can't parse '-32' to u32"));
}

#[test]
fn db_syntax_error() {
    let (_, err) = run(vec![
        "insert 1 user1".into(),
        ".exit".into()
    ]);
    assert!(err[err.len() - 2].contains("[ERROR]syntax error"));
}

#[test]
fn db_table_full() {
    let mut cmds: Vec<String> = (1..7802)
        .map(|i| format!("insert {} user{} person{}@example.com", i, i, i))
        .collect();
    cmds.push(".exit".into());
    
    let (_, err) = run(cmds);
    assert!(err[err.len() - 2].contains("[ERROR]table is full"));
}

#[test]
fn db_too_long() {
    let (_, err) = run(vec![
        "insert 1 nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn abc".into(),
        ".exit".into()
    ]);
    assert!(err[err.len() - 2].contains("[ERROR]'nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn' is too long for username"));
}
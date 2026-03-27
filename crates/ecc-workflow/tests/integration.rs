use std::process::Command;
use std::path::PathBuf;

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove deps/
    path.push("ecc-workflow");
    path
}

fn valid_statuses() -> &'static [&'static str] {
    &["pass", "block", "warn"]
}

/// Parse stdout or stderr as JSON and verify the "status" field is in ["pass","block","warn"]
/// and a "message" field is present.
fn assert_structured_json_output(output: &std::process::Output) {
    // At least one of stdout or stderr should be non-empty JSON
    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");

    // Try stdout first, then stderr
    let json_str = if !stdout.trim().is_empty() {
        stdout.trim()
    } else if !stderr.trim().is_empty() {
        stderr.trim()
    } else {
        panic!(
            "Both stdout and stderr are empty. exit code: {:?}",
            output.status.code()
        );
    };

    let value: serde_json::Value = serde_json::from_str(json_str)
        .unwrap_or_else(|e| panic!("Output is not valid JSON: {e}\nOutput was: {json_str}"));

    let status = value
        .get("status")
        .unwrap_or_else(|| panic!("JSON missing 'status' field: {value}"))
        .as_str()
        .unwrap_or_else(|| panic!("'status' field is not a string: {value}"));

    assert!(
        valid_statuses().contains(&status),
        "'status' value '{status}' is not in {:?}",
        valid_statuses()
    );

    assert!(
        value.get("message").is_some(),
        "JSON missing 'message' field: {value}"
    );
}

#[test]
fn output_is_structured_json() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "ecc-workflow binary not found at {:?}",
        bin
    );

    let temp_dir = tempfile::tempdir().unwrap();

    // Test: init subcommand produces structured JSON
    let output = Command::new(&bin)
        .args(["init", "dev", "test-feature"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .output()
        .expect("failed to execute ecc-workflow init");

    assert_structured_json_output(&output);
}

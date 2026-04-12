//! End-to-end integration test: `hub info` against a loopback fake hub
//! replaying the real captured bytes from `PULSE_PRO_LOCAL_API.md` §6.

mod common;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

/// Verifies that `hub info` sends the expected batch query and renders
/// the hub name from a real hub's captured response. Uses `assert_cmd`
/// to exec the actual compiled binary (not a library call), so this
/// exercises the full clap-parse → tokio-runtime → connect → query →
/// render → exit pipeline end-to-end.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn hub_info_renders_hub_name_from_real_captured_response() {
    let (listener, addr_str) = common::bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        common::serve_one(listener, |mut socket| async move {
            // Batch 1: the client sends hub name + serial + version enum
            // + position enum queries. Read them, then respond.
            let mut buf = [0u8; 256];
            let _ = socket.read(&mut buf).await.unwrap();

            socket
                .write_all(
                    b"!000NAME6217 Shade Hub;!000SN2016197;\
                      !BR1vB10;!4JKvD22;!MWXvD22;!3YCvD22;\
                      !4JKr000b000,R58;!MWXr100b180,R4C;!3YCr000b000,R4C;",
                )
                .await
                .unwrap();
            socket.flush().await.unwrap();

            // Batch 2: the client sends per-motor NAME queries. We need
            // to actually wait for the client to send batch 2 before
            // replying — if we reply too early, the name frames get
            // swallowed by batch 1's read_for(3s) window and are lost.
            let mut buf2 = [0u8; 256];
            let _ = socket.read(&mut buf2).await.unwrap();

            socket
                .write_all(b"!4JKNAMEJohn House;!MWXNAMEDining Room;!3YCNAMEKitchen;")
                .await
                .unwrap();
            socket.flush().await.unwrap();

            // Keep socket open so the client can finish reading batch 2.
            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        })
        .await;
    });

    // Exec the binary via assert_cmd (blocking — needs spawn_blocking).
    let addr_for_cmd = addr_str.clone();
    let output = tokio::task::spawn_blocking(move || {
        assert_cmd::Command::cargo_bin("libiot-rollease-automate-pulse-pro-hub")
            .unwrap()
            .args(["--hub", &addr_for_cmd, "hub", "info"])
            .output()
            .expect("failed to exec CLI binary")
    })
    .await
    .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // The human output should include the hub name.
    assert!(
        stdout.contains("6217 Shade Hub"),
        "expected hub name in output, got: {stdout}"
    );
    // And motor names.
    assert!(
        stdout.contains("Kitchen"),
        "expected 'Kitchen' motor in output"
    );
    assert!(
        stdout.contains("Dining Room"),
        "expected 'Dining Room' motor in output"
    );

    let _ = fake_hub.await;
}

/// Verifies that `--format json hub info` produces valid JSON with the
/// expected top-level fields.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn hub_info_json_output_has_expected_fields() {
    let (listener, addr_str) = common::bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        common::serve_one(listener, |mut socket| async move {
            // Batch 1.
            let mut buf = [0u8; 256];
            let _ = socket.read(&mut buf).await;

            socket
                .write_all(
                    b"!000NAME6217 Shade Hub;!000SN2016197;\
                      !BR1vB10;!4JKvD22;\
                      !4JKr000b000,R58;",
                )
                .await
                .unwrap();
            socket.flush().await.unwrap();

            // Batch 2: wait for client to send name queries.
            let mut buf2 = [0u8; 256];
            let _ = socket.read(&mut buf2).await;

            socket.write_all(b"!4JKNAMEJohn House;").await.unwrap();
            socket.flush().await.unwrap();

            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        })
        .await;
    });

    let addr_for_cmd = addr_str.clone();
    let output = tokio::task::spawn_blocking(move || {
        assert_cmd::Command::cargo_bin("libiot-rollease-automate-pulse-pro-hub")
            .unwrap()
            .args(["--hub", &addr_for_cmd, "--format", "json", "hub", "info"])
            .output()
            .expect("failed to exec CLI binary")
    })
    .await
    .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");

    assert_eq!(json["hub_name"], "6217 Shade Hub");
    assert_eq!(json["hub_serial"], "2016197");
    assert!(json["motors"].is_array());

    let _ = fake_hub.await;
}

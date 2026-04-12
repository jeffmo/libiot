//! End-to-end integration test: motor control commands against a
//! loopback fake hub.

mod common;

/// Verifies that `close 4JK` writes the exact `!4JKc;` bytes on the
/// wire and exits successfully.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn close_subcommand_writes_close_bytes_and_exits_ok() {
    let (listener, addr_str) = common::bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        common::serve_one(listener, |mut socket| async move {
            common::expect_bytes(&mut socket, b"!4JKc;").await;
            // Socket drops here, closing the connection cleanly.
        })
        .await;
    });

    let addr_for_cmd = addr_str.clone();
    let cmd_result = tokio::task::spawn_blocking(move || {
        assert_cmd::Command::cargo_bin("libiot-rollease-automate-pulse-pro-hub")
            .unwrap()
            .args(["--hub", &addr_for_cmd, "close", "4JK"])
            .assert()
            .success();
    })
    .await;

    assert!(cmd_result.is_ok());
    let _ = fake_hub.await;
}

/// Verifies that `set-position MWX 50` writes `!MWXm050;` (the
/// 3-digit-padded move command) on the wire.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn set_position_writes_three_digit_padded_move_command() {
    let (listener, addr_str) = common::bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        common::serve_one(listener, |mut socket| async move {
            common::expect_bytes(&mut socket, b"!MWXm050;").await;
        })
        .await;
    });

    let addr_for_cmd = addr_str.clone();
    let cmd_result = tokio::task::spawn_blocking(move || {
        assert_cmd::Command::cargo_bin("libiot-rollease-automate-pulse-pro-hub")
            .unwrap()
            .args(["--hub", &addr_for_cmd, "set-position", "MWX", "50"])
            .assert()
            .success();
    })
    .await;

    assert!(cmd_result.is_ok());
    let _ = fake_hub.await;
}

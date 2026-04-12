//! The [`AutomatePulseProHub`] public client struct.

use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;
use tokio::sync::Mutex;

use crate::codec::IncomingFrame;
use crate::codec::encode_close;
use crate::codec::encode_close_all;
#[cfg(feature = "dangerous-ops")]
use crate::codec::encode_delete_limits;
#[cfg(feature = "dangerous-ops")]
use crate::codec::encode_delete_motor;
#[cfg(feature = "dangerous-ops")]
use crate::codec::encode_factory_reset;
use crate::codec::encode_jog_close;
use crate::codec::encode_jog_open;
use crate::codec::encode_move_to;
use crate::codec::encode_open;
use crate::codec::encode_open_all;
#[cfg(feature = "dangerous-ops")]
use crate::codec::encode_pair_motor;
use crate::codec::encode_query_hub_name;
use crate::codec::encode_query_hub_serial;
use crate::codec::encode_query_motor_enum;
use crate::codec::encode_query_motor_name;
use crate::codec::encode_query_motor_position;
use crate::codec::encode_query_motor_position_all;
use crate::codec::encode_query_motor_version;
use crate::codec::encode_query_motor_voltage;
#[cfg(feature = "dangerous-ops")]
use crate::codec::encode_set_lower_limit;
#[cfg(feature = "dangerous-ops")]
use crate::codec::encode_set_upper_limit;
use crate::codec::encode_stop;
use crate::codec::encode_stop_all;
use crate::codec::encode_tilt;
#[cfg(feature = "dangerous-ops")]
use crate::codec::encode_unpair_motor;
use crate::error::Error;
use crate::error::Result;
use crate::hub_info::HubInfo;
use crate::motor::Motor;
use crate::motor_address::MotorAddress;
use crate::motor_position::MotorPosition;
use crate::motor_type::MotorType;
use crate::motor_version::MotorVersion;
use crate::motor_voltage::MotorVoltage;
use crate::transport::Transport;

/// Default TCP port of the hub's ASCII protocol endpoint.
pub const DEFAULT_PORT: u16 = 1487;

/// Default timeout for a single query response.
const DEFAULT_QUERY_TIMEOUT: Duration = Duration::from_millis(2_500);

/// Collection window for broadcast query responses — how long to keep
/// reading after the broadcast is sent before assuming every
/// responsive motor has had a chance to reply.
const BROADCAST_COLLECT_WINDOW: Duration = Duration::from_millis(3_000);

/// Async client for the Rollease Acmeda Automate Pulse Pro shade hub's
/// LAN ASCII protocol on TCP port 1487.
///
/// Holds one long-lived TCP connection to the hub and serializes all
/// requests through it (the protocol recommends a single long-lived
/// connection — rapid connect/disconnect cycles occasionally drop
/// queries on the hub side).
///
/// `AutomatePulseProHub` is cheap to clone — the clone shares the same
/// underlying connection via an `Arc<Mutex<_>>`, so you can pass
/// handles to as many async tasks as you like without worrying about
/// the transport.
///
/// # Examples
///
/// ```no_run
/// use libiot_rollease_automate_pulse_pro_hub::AutomatePulseProHub;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let hub = AutomatePulseProHub::connect("192.168.5.234:1487").await?;
/// hub.close_all().await?;
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct AutomatePulseProHub {
    inner: Arc<Mutex<Transport<TcpStream>>>,
}

impl AutomatePulseProHub {
    /// Connect to the hub at `addr` and return a ready-to-use client.
    ///
    /// The hub must have been paired once via the mobile *Automate
    /// Pulse 2* app before it will accept local TCP connections on
    /// port 1487 — factory-fresh hubs do not listen on 1487 until
    /// first-time provisioning is complete (see §1.1 of the in-crate
    /// `PULSE_PRO_LOCAL_API.md`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the TCP connection attempt fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use libiot_rollease_automate_pulse_pro_hub::AutomatePulseProHub;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// // The hub's LAN IP plus the fixed ASCII-protocol port 1487.
    /// let hub = AutomatePulseProHub::connect("192.168.5.234:1487").await?;
    /// # Ok(()) }
    /// ```
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        // Disable Nagle for snappier motor-control latency. The ASCII
        // frames are tiny; bundling them into large packets only adds
        // latency without improving throughput.
        if let Err(err) = stream.set_nodelay(/* nodelay = */ true) {
            return Err(Error::Io(err));
        }
        Ok(Self {
            inner: Arc::new(Mutex::new(Transport::new(stream))),
        })
    }

    /// Wrap an already-connected [`TcpStream`]. Useful for tests that
    /// want to hand a hand-rolled socket to the client.
    #[doc(hidden)]
    pub fn from_stream(stream: TcpStream) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Transport::new(stream))),
        }
    }

    // ---- Hub-level queries --------------------------------------------

    /// Query the hub's friendly name (`!000NAME?;`).
    pub async fn hub_name(&self) -> Result<String> {
        let frames = self
            .query_batch(
                &encode_query_hub_name(),
                |frames| {
                    frames
                        .iter()
                        .any(|f| matches!(f, IncomingFrame::HubName(_)))
                },
                DEFAULT_QUERY_TIMEOUT,
            )
            .await?;

        for frame in frames {
            if let IncomingFrame::HubName(name) = frame {
                return Ok(name);
            }
        }
        Err(Error::Timeout {
            ms: millis(DEFAULT_QUERY_TIMEOUT),
        })
    }

    /// Query the hub's serial number (`!000SN?;`).
    pub async fn hub_serial(&self) -> Result<String> {
        let frames = self
            .query_batch(
                &encode_query_hub_serial(),
                |frames| {
                    frames
                        .iter()
                        .any(|f| matches!(f, IncomingFrame::HubSerial(_)))
                },
                DEFAULT_QUERY_TIMEOUT,
            )
            .await?;

        for frame in frames {
            if let IncomingFrame::HubSerial(sn) = frame {
                return Ok(sn);
            }
        }
        Err(Error::Timeout {
            ms: millis(DEFAULT_QUERY_TIMEOUT),
        })
    }

    /// Query full hub info — name, serial, and every paired motor — in
    /// one batched round-trip.
    ///
    /// Sends a single concatenated write containing the hub name, hub
    /// serial, version enumeration, and position enumeration queries.
    /// Then sends a second batched write of per-motor name queries.
    /// Returns a [`HubInfo`] with every paired motor (hub gateway
    /// filtered out), each populated with name, version, and position
    /// if the hub responded with one.
    pub async fn info(&self) -> Result<HubInfo> {
        // First batch: hub name + hub serial + version enum + position enum.
        let mut batch1 = Vec::new();
        batch1.extend_from_slice(&encode_query_hub_name());
        batch1.extend_from_slice(&encode_query_hub_serial());
        batch1.extend_from_slice(&encode_query_motor_enum());
        batch1.extend_from_slice(&encode_query_motor_position_all());

        let first_frames = {
            let mut inner = self.inner.lock().await;
            inner.write_frames(&batch1).await?;
            inner.read_for(BROADCAST_COLLECT_WINDOW).await?
        };

        // Assemble the partial info from the first batch.
        let mut hub_name: Option<String> = None;
        let mut hub_serial: Option<String> = None;
        let mut motor_addresses: Vec<MotorAddress> = Vec::new();
        let mut motor_versions: Vec<(MotorAddress, MotorType, String)> = Vec::new();
        let mut motor_positions: Vec<(MotorAddress, MotorPosition)> = Vec::new();

        for frame in first_frames {
            match frame {
                IncomingFrame::HubName(name) => hub_name = Some(name),
                IncomingFrame::HubSerial(sn) => hub_serial = Some(sn),
                IncomingFrame::MotorVersionRec {
                    addr,
                    motor_type,
                    version,
                } => {
                    if !motor_type.is_hub_gateway() {
                        if !motor_addresses.contains(&addr) {
                            motor_addresses.push(addr);
                        }
                        motor_versions.push((addr, motor_type, version));
                    }
                },
                IncomingFrame::MotorPositionRec { addr, position } => {
                    motor_positions.push((addr, position));
                },
                // Hub errors, motor names, etc. are unexpected in the first
                // batch but don't fail the info — they're just ignored.
                _ => {},
            }
        }

        // Second batch: per-motor friendly name queries.
        let motor_names: Vec<(MotorAddress, String)> = if motor_addresses.is_empty() {
            Vec::new()
        } else {
            let mut batch2 = Vec::new();
            for addr in &motor_addresses {
                batch2.extend_from_slice(&encode_query_motor_name(addr));
            }

            let name_frames = {
                let mut inner = self.inner.lock().await;
                inner.write_frames(&batch2).await?;
                inner.read_for(BROADCAST_COLLECT_WINDOW).await?
            };

            name_frames
                .into_iter()
                .filter_map(|frame| match frame {
                    IncomingFrame::MotorName { addr, name } => Some((addr, name)),
                    _ => None,
                })
                .collect()
        };

        // Stitch together the final `Motor` list in the order the hub
        // reported the version records.
        let motors = motor_addresses
            .iter()
            .filter_map(|addr| {
                let (_, motor_type, version) = motor_versions.iter().find(|(a, _, _)| a == addr)?;
                let position = motor_positions
                    .iter()
                    .find(|(a, _)| a == addr)
                    .map(|(_, p)| *p);
                let name = motor_names
                    .iter()
                    .find(|(a, _)| a == addr)
                    .map(|(_, n)| n.clone());
                Some(Motor {
                    address: *addr,
                    name,
                    version: MotorVersion {
                        address: *addr,
                        motor_type: *motor_type,
                        version: version.clone(),
                    },
                    position,
                })
            })
            .collect();

        Ok(HubInfo {
            hub_name: hub_name.unwrap_or_default(),
            hub_serial: hub_serial.unwrap_or_default(),
            motors,
        })
    }

    /// Return just the motors from a fresh [`AutomatePulseProHub::info`].
    pub async fn list_motors(&self) -> Result<Vec<Motor>> {
        Ok(self.info().await?.motors)
    }

    // ---- Per-motor queries --------------------------------------------

    /// Query a specific motor's friendly name (`!<addr>NAME?;`).
    pub async fn motor_name(&self, addr: &MotorAddress) -> Result<String> {
        let bytes = encode_query_motor_name(addr);
        let target = *addr;
        let frames = self
            .query_batch(
                &bytes,
                |frames| {
                    frames.iter().any(|f| {
                        matches!(f, IncomingFrame::MotorName { addr, .. } if *addr == target)
                            || matches!(f, IncomingFrame::HubError { addr, .. } if *addr == target)
                    })
                },
                DEFAULT_QUERY_TIMEOUT,
            )
            .await?;
        extract_motor_name(frames, addr)
    }

    /// Query a specific motor's current position (`!<addr>r?;`).
    pub async fn motor_position(&self, addr: &MotorAddress) -> Result<MotorPosition> {
        let bytes = encode_query_motor_position(addr);
        let target = *addr;
        let frames = self
            .query_batch(
                &bytes,
                |frames| {
                    frames.iter().any(|f| {
                        matches!(f, IncomingFrame::MotorPositionRec { addr, .. } if *addr == target)
                            || matches!(f, IncomingFrame::HubError { addr, .. } if *addr == target)
                    })
                },
                DEFAULT_QUERY_TIMEOUT,
            )
            .await?;
        extract_motor_position(frames, addr)
    }

    /// Query a specific motor's firmware version (`!<addr>v?;`).
    pub async fn motor_version(&self, addr: &MotorAddress) -> Result<MotorVersion> {
        let bytes = encode_query_motor_version(addr);
        let target = *addr;
        let frames = self
            .query_batch(
                &bytes,
                |frames| {
                    frames.iter().any(|f| {
                        matches!(f, IncomingFrame::MotorVersionRec { addr, .. } if *addr == target)
                            || matches!(f, IncomingFrame::HubError { addr, .. } if *addr == target)
                    })
                },
                DEFAULT_QUERY_TIMEOUT,
            )
            .await?;
        extract_motor_version(frames, addr)
    }

    /// Query a specific motor's battery voltage (`!<addr>pVc?;`).
    pub async fn motor_voltage(&self, addr: &MotorAddress) -> Result<MotorVoltage> {
        let bytes = encode_query_motor_voltage(addr);
        let target = *addr;
        let frames = self
            .query_batch(
                &bytes,
                |frames| {
                    frames.iter().any(|f| {
                        matches!(f, IncomingFrame::MotorVoltageRec { addr, .. } if *addr == target)
                            || matches!(f, IncomingFrame::HubError { addr, .. } if *addr == target)
                    })
                },
                DEFAULT_QUERY_TIMEOUT,
            )
            .await?;
        extract_motor_voltage(frames, addr)
    }

    // ---- Motor control ------------------------------------------------

    /// Open a specific motor (lift to 0% closed).
    pub async fn open(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_open(addr)).await
    }

    /// Close a specific motor (lift to 100% closed).
    pub async fn close(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_close(addr)).await
    }

    /// Stop any in-flight motion on a specific motor.
    pub async fn stop(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_stop(addr)).await
    }

    /// Move a specific motor to a given closed-lift percentage
    /// (0..=100).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPercentage`] if `percent` is greater
    /// than 100.
    pub async fn set_position(&self, addr: &MotorAddress, percent: u8) -> Result<()> {
        let bytes = encode_move_to(addr, percent)?;
        self.fire_and_forget(&bytes).await
    }

    /// Tilt a specific motor to a given percentage (0..=100).
    ///
    /// "Tilt" refers to the angle of individual slats on venetian
    /// blinds (horizontal blinds) or shutters: `0` = slats
    /// horizontal/open (maximum light), `100` = slats fully
    /// angled/closed (minimum light). Tilt only applies to slatted
    /// shades — roller shades, cellular shades, and drapes silently
    /// ignore this command.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPercentage`] if `percent` is greater
    /// than 100.
    pub async fn set_tilt(&self, addr: &MotorAddress, percent: u8) -> Result<()> {
        let bytes = encode_tilt(addr, percent)?;
        self.fire_and_forget(&bytes).await
    }

    /// Nudge a specific motor one step open (`!<addr>oA;`).
    pub async fn jog_open(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_jog_open(addr)).await
    }

    /// Nudge a specific motor one step closed (`!<addr>cA;`).
    pub async fn jog_close(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_jog_close(addr)).await
    }

    // ---- Broadcast ----------------------------------------------------

    /// Open every paired motor (`!000o;`).
    pub async fn open_all(&self) -> Result<()> {
        self.fire_and_forget(&encode_open_all()).await
    }

    /// Close every paired motor (`!000c;`).
    pub async fn close_all(&self) -> Result<()> {
        self.fire_and_forget(&encode_close_all()).await
    }

    /// Stop every paired motor (`!000s;`).
    pub async fn stop_all(&self) -> Result<()> {
        self.fire_and_forget(&encode_stop_all()).await
    }

    // ---- dangerous-ops ------------------------------------------------

    /// Pair a new motor. The hub auto-assigns a random 3-char address.
    #[cfg(feature = "dangerous-ops")]
    pub async fn pair_motor(&self) -> Result<()> {
        self.fire_and_forget(&encode_pair_motor()).await
    }

    /// Unpair a motor — requires the motor to acknowledge.
    #[cfg(feature = "dangerous-ops")]
    pub async fn unpair_motor(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_unpair_motor(addr)).await
    }

    /// Delete a motor from the hub's paired list. Does not require a
    /// motor ack.
    #[cfg(feature = "dangerous-ops")]
    pub async fn delete_motor(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_delete_motor(addr)).await
    }

    /// Factory-reset the hub — wipes all pairing configuration.
    #[cfg(feature = "dangerous-ops")]
    pub async fn factory_reset(&self) -> Result<()> {
        self.fire_and_forget(&encode_factory_reset()).await
    }

    /// Set a motor's upper limit at its current position.
    #[cfg(feature = "dangerous-ops")]
    pub async fn set_upper_limit(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_set_upper_limit(addr)).await
    }

    /// Set a motor's lower limit at its current position.
    #[cfg(feature = "dangerous-ops")]
    pub async fn set_lower_limit(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_set_lower_limit(addr)).await
    }

    /// Delete all of a motor's configured limits.
    #[cfg(feature = "dangerous-ops")]
    pub async fn delete_limits(&self, addr: &MotorAddress) -> Result<()> {
        self.fire_and_forget(&encode_delete_limits(addr)).await
    }

    // ---- internal helpers ---------------------------------------------

    /// Send one or more concatenated frames and wait for responses. The
    /// `is_done` closure decides when we have enough.
    async fn query_batch<F>(
        &self,
        bytes: &[u8],
        is_done: F,
        timeout: Duration,
    ) -> Result<Vec<IncomingFrame>>
    where
        F: FnMut(&[IncomingFrame]) -> bool,
    {
        let mut inner = self.inner.lock().await;
        inner.write_frames(bytes).await?;
        inner.read_until(is_done, timeout).await
    }

    /// Send a command that doesn't expect a response frame.
    ///
    /// Motion commands (`open`, `close`, `stop`, `set_position`) don't
    /// ACK when the target motor has no limits set (see §2.9 quirk #4
    /// of `PULSE_PRO_LOCAL_API.md`), so we never wait for a response.
    /// Callers that want confirmation poll [`Self::motor_position`]
    /// themselves.
    async fn fire_and_forget(&self, bytes: &[u8]) -> Result<()> {
        let mut inner = self.inner.lock().await;
        inner.write_frames(bytes).await
    }
}

// ---- response extractors --------------------------------------------------

fn extract_motor_name(frames: Vec<IncomingFrame>, want: &MotorAddress) -> Result<String> {
    for frame in frames {
        match frame {
            IncomingFrame::MotorName { addr, name } if addr == *want => return Ok(name),
            IncomingFrame::HubError { addr, code } if addr == *want => {
                return Err(Error::HubError {
                    address: addr,
                    code,
                });
            },
            _ => {},
        }
    }
    Err(Error::Timeout {
        ms: millis(DEFAULT_QUERY_TIMEOUT),
    })
}

fn extract_motor_position(
    frames: Vec<IncomingFrame>,
    want: &MotorAddress,
) -> Result<MotorPosition> {
    for frame in frames {
        match frame {
            IncomingFrame::MotorPositionRec { addr, position } if addr == *want => {
                return Ok(position);
            },
            IncomingFrame::HubError { addr, code } if addr == *want => {
                return Err(Error::HubError {
                    address: addr,
                    code,
                });
            },
            _ => {},
        }
    }
    Err(Error::Timeout {
        ms: millis(DEFAULT_QUERY_TIMEOUT),
    })
}

fn extract_motor_version(frames: Vec<IncomingFrame>, want: &MotorAddress) -> Result<MotorVersion> {
    for frame in frames {
        match frame {
            IncomingFrame::MotorVersionRec {
                addr,
                motor_type,
                version,
            } if addr == *want => {
                return Ok(MotorVersion {
                    address: addr,
                    motor_type,
                    version,
                });
            },
            IncomingFrame::HubError { addr, code } if addr == *want => {
                return Err(Error::HubError {
                    address: addr,
                    code,
                });
            },
            _ => {},
        }
    }
    Err(Error::Timeout {
        ms: millis(DEFAULT_QUERY_TIMEOUT),
    })
}

fn extract_motor_voltage(frames: Vec<IncomingFrame>, want: &MotorAddress) -> Result<MotorVoltage> {
    for frame in frames {
        match frame {
            IncomingFrame::MotorVoltageRec { addr, voltage } if addr == *want => {
                return Ok(voltage);
            },
            IncomingFrame::HubError { addr, code } if addr == *want => {
                return Err(Error::HubError {
                    address: addr,
                    code,
                });
            },
            _ => {},
        }
    }
    Err(Error::Timeout {
        ms: millis(DEFAULT_QUERY_TIMEOUT),
    })
}

fn millis(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(feature = "signal")]
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "chat-isomorphic", about = "isomorphic chat layer over many messengers")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Signal backend (presage). Only compiled in when `--features signal`.
    #[cfg(feature = "signal")]
    Signal {
        #[arg(
            long,
            global = true,
            env = "SIGNAL_STORE_PATH",
            default_value = ".data/signal/signal.db"
        )]
        store: PathBuf,

        #[command(subcommand)]
        verb: SignalVerb,
    },
}

#[cfg(feature = "signal")]
#[derive(Subcommand)]
enum SignalVerb {
    /// Print own identity (ACI / PNI / E.164) from the local store.
    Whoami,
    /// Link this machine as a fresh secondary device.
    /// Store path must not already exist — `rm -rf .data/signal/` first if
    /// relinking.
    Link {
        #[arg(long, default_value = "chat-isomorphic-dev")]
        device_name: String,
    },
    /// What devices does the primary consider linked?
    ListDevices,
    /// Remove a linked device by id. Note: presage rejects this from a
    /// secondary; use the iOS / Desktop Signal app instead.
    Unlink {
        #[arg(long)]
        device_id: i64,
    },
    /// Drive Manager::receive_messages and print every decoded event for
    /// `--seconds`. Exits early on BacklogDrained if `--stop-when-empty`.
    Events {
        #[arg(long, default_value_t = 30)]
        seconds: u64,
        #[arg(long)]
        stop_when_empty: bool,
    },
    /// Send text to your own ACI (Note-to-Self).
    SendSelf {
        body: String,
    },
    /// List contacts cached in the local store. Empty until the primary
    /// pushes contact-sync — use `request-contacts` to ask for it.
    ListContacts,
    /// List groups cached in the local store.
    ListGroups,
    /// Send a SyncMessage::Request{Contacts} to the primary, asking it
    /// to push the contact list back to us. Then run `events` to see
    /// whether the response arrives and gets stored.
    RequestContacts,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
    dispatch(Cli::parse()).await
}

#[cfg(feature = "signal")]
async fn dispatch(cli: Cli) -> anyhow::Result<()> {
    match cli.cmd {
        Cmd::Signal { store, verb } => run_signal(store, verb).await,
    }
}

#[cfg(not(feature = "signal"))]
async fn dispatch(_cli: Cli) -> anyhow::Result<()> {
    anyhow::bail!("no backends compiled in — build with --features signal")
}

#[cfg(feature = "signal")]
async fn run_signal(store: PathBuf, verb: SignalVerb) -> anyhow::Result<()> {
    use chat_isomorphic_backend_signal::SignalBackend;
    use chat_isomorphic_core::{Backend, ThreadId};
    use futures::StreamExt;

    // Link is the only verb that runs against an unregistered store.
    if let SignalVerb::Link { device_name } = verb {
        return link_flow(&store, device_name).await;
    }

    let mut backend = SignalBackend::open(&store).await?;

    match verb {
        SignalVerb::Link { .. } => unreachable!(),
        SignalVerb::Whoami => {
            let id = backend.whoami().await?;
            println!("aci   {}", id.aci);
            if let Some(pni) = id.pni {
                println!("pni   {}", pni);
            }
            if let Some(e164) = id.e164 {
                println!("e164  {}", e164);
            }
        }
        SignalVerb::ListDevices => {
            let me = backend.device_id();
            let devices = backend.list_devices().await?;
            for d in &devices {
                let marker = if d.id == me { " (this device)" } else { "" };
                let name = d.name.as_deref().unwrap_or("(no device name)");
                println!(
                    "- Device {}{}\n  Name: {}\n  Created: {}\n  Last seen: {}",
                    d.id, marker, name, d.created_at, d.last_seen
                );
            }
            println!("{} device(s) linked", devices.len());
        }
        SignalVerb::ListContacts => {
            let contacts = backend.list_contacts().await?;
            for c in &contacts {
                let phone = c
                    .phone_number
                    .as_ref()
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "(no e164)".into());
                let name = if c.name.is_empty() {
                    "(no name)"
                } else {
                    c.name.as_str()
                };
                println!("- {}\n  Name:  {}\n  Phone: {}", c.uuid, name, phone);
            }
            println!("{} contact(s) cached", contacts.len());
        }
        SignalVerb::ListGroups => {
            let groups = backend.list_groups().await?;
            for (master_key, g) in &groups {
                let short = master_key
                    .iter()
                    .take(8)
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                let title = if g.title.is_empty() {
                    "(no title)"
                } else {
                    g.title.as_str()
                };
                println!(
                    "- {}…\n  Title:    {}\n  Members:  {}\n  Revision: {}",
                    short,
                    title,
                    g.members.len(),
                    g.revision
                );
            }
            println!("{} group(s) cached", groups.len());
        }
        SignalVerb::RequestContacts => {
            backend.request_contacts().await?;
            println!("contact-sync request sent — run `signal events` to see the response");
        }
        SignalVerb::Unlink { device_id } => {
            backend.unlink_secondary(device_id).await?;
            println!("unlinked device id={device_id}");
        }
        SignalVerb::Events { seconds, stop_when_empty } => {
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(seconds);
            let mut stream = backend.events().await?;
            loop {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                if remaining.is_zero() {
                    break;
                }
                tokio::select! {
                    _ = tokio::time::sleep_until(deadline) => break,
                    next = stream.next() => match next {
                        Some(evt) => {
                            println!("{evt:#?}");
                            if stop_when_empty
                                && matches!(evt, chat_isomorphic_core::Event::BacklogDrained)
                            {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
        SignalVerb::SendSelf { body } => {
            let me = backend.whoami().await?.aci;
            let ts = backend.send_text(&ThreadId::Contact(me), &body).await?;
            println!("sent ts={ts}");
        }
    }

    Ok(())
}

#[cfg(feature = "signal")]
async fn link_flow(store: &std::path::Path, device_name: String) -> anyhow::Result<()> {
    use chat_isomorphic_backend_signal::SignalBackend;
    use chat_isomorphic_core::Backend;

    if store.exists() {
        anyhow::bail!(
            "store path {} already exists — remove it first if you intend to relink",
            store.display()
        );
    }
    if let Some(parent) = store.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let png = store.with_file_name("qr.png");
    let png_clone = png.clone();
    let backend = SignalBackend::link(store, device_name, move |provision_url| {
        let url_str = provision_url.to_string();
        match std::process::Command::new("qrencode")
            .args(["-o"])
            .arg(&png_clone)
            .args(["-s", "10", "-m", "4", &url_str])
            .status()
        {
            Ok(s) if s.success() => {
                println!("QR rendered: {}", png_clone.display());
                println!("URL fallback: {url_str}");
                let _ = std::process::Command::new("open").arg(&png_clone).status();
                println!("Scan from your phone: Signal → Settings → Linked Devices → +");
            }
            _ => {
                println!("qrencode failed; scan this URL with another QR generator:\n  {url_str}");
            }
        }
    })
    .await?;

    let id = backend.whoami().await?;
    println!("\nlinked successfully");
    println!("aci   {}", id.aci);
    if let Some(e164) = id.e164 {
        println!("e164  {}", e164);
    }
    Ok(())
}

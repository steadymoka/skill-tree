use std::env;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

fn standalone_dir(base: &Path) -> PathBuf {
    base.join(".next").join("standalone")
}

/// Ensure `.next/static` and `public` are accessible from the standalone dir.
/// Next.js standalone output excludes static assets by design.
fn ensure_static_assets(root: &Path) {
    let standalone = standalone_dir(root);

    let static_link = standalone.join(".next").join("static");
    let static_src = root.join(".next").join("static");
    if !static_link.exists() && static_src.exists() {
        let _ = unix_fs::symlink(&static_src, &static_link);
    }

    let public_link = standalone.join("public");
    let public_src = root.join("public");
    if !public_link.exists() && public_src.exists() {
        let _ = unix_fs::symlink(&public_src, &public_link);
    }
}

/// Start the Next.js standalone server and open the browser.
pub fn start_web(project_root: Option<PathBuf>) -> Result<()> {
    let root = match project_root {
        Some(p) => p,
        None => infer_project_root()?,
    };

    ensure_static_assets(&root);

    let server_js = standalone_dir(&root).join("server.js");
    let port = find_available_port(3000)?;
    let url = format!("http://127.0.0.1:{}", port);

    println!("\n  Skill Tree web UI starting at {}\n", url);

    let mut child = Command::new("node")
        .arg(&server_js)
        .env("PORT", port.to_string())
        .env("HOSTNAME", "127.0.0.1")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| {
            if !server_js.exists() {
                format!(
                    "Next.js standalone server not found at {}.\nRun 'pnpm build' first.",
                    server_js.display()
                )
            } else {
                "failed to start node — is Node.js installed?".to_string()
            }
        })?;

    if wait_for_server(port, Duration::from_secs(10)) {
        if let Err(e) = open::that(&url) {
            eprintln!("  Could not open browser: {}. Open {} manually.", e, url);
        }
    } else {
        eprintln!("  Server did not start within 10s. Open {} manually.", url);
    }

    child.wait().context("node process exited unexpectedly")?;
    Ok(())
}

fn find_available_port(start: u16) -> Result<u16> {
    for port in start..start + 100 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }
    bail!("no available port found in range {}–{}", start, start + 99)
}

fn wait_for_server(port: u16, timeout: Duration) -> bool {
    let start = Instant::now();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut delay = Duration::from_millis(20);
    while start.elapsed() < timeout {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_ok() {
            return true;
        }
        std::thread::sleep(delay);
        delay = (delay * 2).min(Duration::from_millis(200));
    }
    false
}

/// Walk up from CWD (then from binary location) to find the project root.
fn infer_project_root() -> Result<PathBuf> {
    let mut dir = env::current_dir().context("cannot determine current directory")?;
    loop {
        if standalone_dir(&dir).exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }

    if let Ok(exe) = env::current_exe() {
        for ancestor in exe.ancestors().skip(1) {
            if standalone_dir(ancestor).exists() {
                return Ok(ancestor.to_path_buf());
            }
        }
    }

    bail!("cannot find project root with .next/standalone/. Pass --root or cd into the project.")
}

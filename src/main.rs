use anyhow::Result;

use replit_xray::server::Server;

fn main() -> Result<()> {
    let mut app = Server::new();

    app.run()?;

    Ok(())
}

use anyhow::Result;

use replit_xray::server::Server;

fn main() -> Result<()> {
    let app = Server::new();

    app.run()?;
    Ok(())
}

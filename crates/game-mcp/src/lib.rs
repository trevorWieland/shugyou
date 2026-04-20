use anyhow::Result;

#[derive(Debug, Default)]
pub struct McpScaffold {
    pub command_catalog: Vec<String>,
}

impl McpScaffold {
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_catalog: Vec::new(),
        }
    }

    pub fn start(&self) -> Result<()> {
        println!(
            "game-mcp scaffold ready; command catalog size = {}",
            self.command_catalog.len()
        );
        Ok(())
    }
}

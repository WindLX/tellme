mod app;
mod capture;
mod config;
mod filter;

fn main() -> anyhow::Result<()> {
    crate::app::app()
}

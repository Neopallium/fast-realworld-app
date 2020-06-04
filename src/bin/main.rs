use dotenv::dotenv;

use fast_realworld::{app::*, error::*};

fn main() -> Result<()> {
  dotenv().ok();
  env_logger::init();

  let yaml = clap::load_yaml!("main-cli.yml");
  let cli = clap::App::from(yaml).get_matches();

  let config = AppConfig::new_clap(&cli)?;

  match cli.subcommand_name() {
    // default to 'serve' command.
    _ => serve::execute(config)?,
  }
  log::info!("Main finished");
  Ok(())
}


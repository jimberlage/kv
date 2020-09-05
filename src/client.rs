use clap::Clap;

#[derive(Clap)]
pub struct Opts {
    #[clap(short, long, default_value = "localhost")]
    host: String,
    #[clap(short, long, default_value = "60054")]
    port: u16,
}

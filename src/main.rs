mod cli;
mod money;
mod engine;
mod processing;
mod transactions;
use std::error::Error;

use std::io::stdout;
use simple_logger::SimpleLogger;
use engine::ToyPaymentsEngine;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::from_args();
    let log_level = match args.verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    let mut engine = ToyPaymentsEngine::new();

    let mut rdr = csv::Reader::from_path(args.tx_csv_path)?;
    rdr.deserialize().filter_map(Result::ok).for_each(|tx| engine.process(tx));

    let mut wrt = csv::Writer::from_writer(stdout());
    wrt.write_record(&["client", "available", "held", "total", "locked"])?;

    log::debug!("client assets: {:?}", engine.assets());
    engine.assets().iter().for_each(|(client_id, assets)| {
        log::trace!("seriallizing client: <client_td: {}>", client_id);
        wrt.serialize((
            client_id,
            assets.available,
            assets.held,
            assets.total(),
            assets.locked,
        )).expect("failed to seriallize assets to csv");
    });
    wrt.flush()?;

    Ok(())
}

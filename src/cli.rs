use eyre::Result;

/// Parses command line arguments and returns (contract_path, contract_name, max_steps)
pub fn parse_args() -> Result<(String, String, usize)> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        Err(eyre::eyre!(
            "Usage: {} <contract_path> <contract_name> [max_steps]",
            args[0]
        ))?;
    }

    let contract_path = args[1].clone();
    let contract_name = args[2].clone();
    let max_steps = if args.len() > 3 { &args[3] } else { "100" };
    let max_steps = max_steps
        .parse::<usize>()
        .map_err(|_| eyre::eyre!("Max steps argument must be a positive integer"))?;

    Ok((contract_path, contract_name, max_steps))
}

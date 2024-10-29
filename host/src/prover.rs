mod handler;
mod server;
// use std::env;
// use std::fs;
use dotenv::dotenv;

macro_rules! env_var {
    ($var:ident, $key:expr) => {
        let $var = std::env::var($key).expect(&format!("{} is not set", $key));
    };
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    env_var!(generator, "GENERATOR_ADDRESS");
    env_var!(gas_key, "GAS_KEY");
    env_var!(market_id, "MARKET_ID");
    env_var!(http_rpc_url, "HTTP_RPC_URL");
    env_var!(proof_market_place, "PROOF_MARKETPLACE_ADDRESS");
    env_var!(generator_registry, "GENERATOR_REGISTRY_ADDRESS");
    env_var!(start_block, "START_BLOCK");
    env_var!(chain_id, "CHAIN_ID");
    env_var!(max_parallel_proofs, "MAX_PARALLEL_PROOFS");
    env_var!(ivs_url, "IVS_URL");
    env_var!(prover_url, "PROVER_URL");

    let port: u16 = prover_url.parse().unwrap();

    let mut handles = vec![];

    let handle_1 = tokio::spawn(async {

        let start_block: u64 = start_block.parse().expect("Can not parse start_block");
        let chain_id: u64 = chain_id.parse().expect("Can not parse chain _id");
        let max_parallel_proofs: usize = max_parallel_proofs.parse().unwrap_or_else(|_| 1);
        log::info!("Start Block: {}, Max Parallel Requests: {}", start_block.clone(), max_parallel_proofs.clone());

        let listener =
        kalypso_listener::job_creator::JobCreator::simple_listener_for_non_confidential_prover(
            generator,
            market_id.into(),
            http_rpc_url.into(),
            gas_key,
            proof_market_place.into(),
            generator_registry.into(),
            start_block,
            chain_id,
            prover_url,
            ivs_url,
            false,
            max_parallel_proofs,
        );

        listener.run().await
    });
    handles.push(handle_1);

    let handle_2 = tokio::spawn(server::ProvingServer::new(port).start_server());
    handles.push(handle_2);

    for handle in handles {
        let _ = handle.await;
    }

    println!("All tasks completed or shutdown.");

    Ok(())
}

#[cfg(test)]
mod tests {}
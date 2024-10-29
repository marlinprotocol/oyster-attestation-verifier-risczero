use actix_web::web::Data;
use actix_web::{get, http::StatusCode, post, web, HttpResponse, Responder};
use ethers;
use clap::Parser;
use methods::{GUEST_ELF, GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use kalypso_helper::response::response;
use serde_json::{Error, Value};
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,
}

// Get generator status from the supervisord
#[get("/test")]
async fn test() -> impl Responder {
    response(
        "The Risc0 prover is running!!",
        StatusCode::OK,
        Some("Risc0 Prover is running!".into()),
    )
}

#[post("/generateProof")]
async fn generate_proof(
    payload: web::Json<Args>,
) -> impl Responder {
    log::info!("Request received by the risc0 prover");

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    println!(
        "GUEST: 0x{}",
        hex::encode(GUEST_ID.map(u32::to_le_bytes).as_flattened())
    );

    // let args = Args::parse();
    let args: Args = serde_json::from(payload.0);

    // Query attestation from the given url
    let mut attestation = Vec::new();
    ureq::get(&args.url)
        .call()
        .unwrap()
        .into_reader()
        .read_to_end(&mut attestation)
        .unwrap();

    println!("Attestation size: {}", attestation.len());

    let env = ExecutorEnv::builder()
        .write_slice(&attestation)
        .build()
        .unwrap();

    let prover = default_prover();
    // Enable groth16
    let prove_info = prover
        .prove_with_opts(env, GUEST_ELF, &ProverOpts::groth16())
        .unwrap();

    let receipt = prove_info.receipt;

    println!("{:?}", receipt);

    let seal = receipt.inner.groth16().unwrap().seal;
    let image_id = GUEST_ID.map(u32::to_le_bytes).as_flattened();
    let journal = receipt.journal.bytes;

    let value = vec![
        ethers::abi::Token::Bytes(seal),
        ethers::abi::Token::Bytes(image_id),
        ethers::abi::Token::Bytes(journal),
    ];
    let encoded = ethers::abi::encode(&value);
    return Ok(HttpResponse::Ok().json(
        kalypso_generator_models::models::GenerateProofResponse {
            proof: encoded.to_vec(),
        },
    ));
}

// Routes
pub fn routes(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(test)
        .service(generate_proof);
    conf.service(scope);
}
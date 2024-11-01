use actix_web::{get, http::StatusCode, post, web, HttpResponse, Responder};
use ethers;
use clap::Parser;
use methods::{GUEST_ELF, GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use kalypso_helper::response::response;
// use serde_json::{Error, Value};
// use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Debug, Deserialize, Clone)]
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
    payload: web::Json<kalypso_generator_models::models::InputPayload>
) -> impl Responder {
    log::info!("Request received by the risc0 prover");

    // Query attestation from the given url
    let attestation = payload.get_public();

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

    // let seal = receipt.inner.groth16().unwrap().seal.clone(); 
    
    // prefix 50bd1769 bytes to seal, og seal wont work on contracts
    let seal_with_prefix: Vec<u8> = vec![0x50, 0xBD, 0x17, 0x69].into_iter().chain(receipt.inner.groth16().unwrap().seal.clone()).collect();
    let guest = GUEST_ID.map(u32::to_le_bytes);
    let image_id = guest.as_flattened();
    let journal = receipt.journal.bytes;

    let value = vec![
        ethers::abi::Token::Bytes(seal_with_prefix),
        ethers::abi::Token::FixedBytes(image_id.to_vec()),
        ethers::abi::Token::Bytes(journal),
    ];
    let encoded = ethers::abi::encode(&value);
    return HttpResponse::Ok().json(
        kalypso_generator_models::models::GenerateProofResponse {
            proof: encoded.to_vec(),
        },
    );
}

// Routes
pub fn routes(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(test)
        .service(generate_proof);
    conf.service(scope);
}
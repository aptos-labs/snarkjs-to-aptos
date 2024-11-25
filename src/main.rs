use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use ark_bn254::{Fq, Fq2, Fr, G1Projective, G2Projective};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use ark_ff::PrimeField;
use num_bigint::BigUint;
use num_traits::Num;
use hex;

fn into_aptos_move_bytes_expr<T: ark_serialize::CanonicalSerialize>(value: T) -> String {
    let mut buf = vec![];
    value.serialize_compressed(&mut buf).unwrap();
    format!("x\"{}\"", hex::encode(buf))
}

fn into_aptos_move_bytes_arr_expr<T: ark_serialize::CanonicalSerialize>(values: Vec<T>) -> String {
    let mut items = vec![];
    for value in values {
        let bytes = into_aptos_move_bytes_expr(value);
        items.push(bytes);
    }
    let items_str = items.join(",");
    format!("vector[{}]", items_str)
}

type SnarkJsFrRepr = String;
fn as_fr(repr: &SnarkJsFqRepr) -> Fr {
    let bytes = BigUint::from_str_radix(repr.as_str(), 10).unwrap();
    Fr::from_be_bytes_mod_order(&bytes.to_bytes_be())
}

type SnarkJsFqRepr = String;
fn as_fq(repr: &SnarkJsFqRepr) -> Fq {
    let bytes = BigUint::from_str_radix(repr.as_str(), 10).unwrap();
    Fq::from_be_bytes_mod_order(&bytes.to_bytes_be())
}

type SnarkJsFq2Repr = [SnarkJsFqRepr; 2];
fn as_fq2(repr: &SnarkJsFq2Repr) -> Fq2 {
    let x = as_fq(&repr[0]);
    let y = as_fq(&repr[1]);
    Fq2::new(x, y)
}

type SnarkJsG1Repr = [SnarkJsFqRepr; 3];
fn as_g1_proj(repr: &SnarkJsG1Repr) -> G1Projective {
    let a = as_fq(&repr[0]);
    let b = as_fq(&repr[1]);
    let c = as_fq(&repr[2]);
    G1Projective::new(a, b, c)
}

type SnarkJsG2Repr = [SnarkJsFq2Repr; 3];
fn as_g2_proj(repr: &SnarkJsG2Repr) -> G2Projective {
    let a = as_fq2(&repr[0]);
    let b = as_fq2(&repr[1]);
    let c = as_fq2(&repr[2]);
    G2Projective::new(a, b, c)
}

#[derive(Deserialize, Serialize)]
struct SnarkJsGroth16VerificationKey {
    vk_alpha_1: SnarkJsG1Repr,
    vk_beta_2: SnarkJsG2Repr,
    vk_gamma_2: SnarkJsG2Repr,
    vk_delta_2: SnarkJsG2Repr,
    #[serde(rename = "IC")]
    ic: Vec<SnarkJsG1Repr>,
}

#[derive(Deserialize, Serialize)]
struct SnarkJsGroth16Proof {
    pi_a: SnarkJsG1Repr,
    pi_b: SnarkJsG2Repr,
    pi_c: SnarkJsG1Repr,
}

type SnarkJsGroth16PublicInput = Vec<SnarkJsFrRepr>;

fn read_file_to_string(file_path: &str) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn copy_dir_all(src: &Path, dest: &Path) -> Result<()> {
    if !dest.exists() {
        fs::create_dir_all(dest)?; // Create missing directories
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if entry_path.is_dir() {
            // Recursively copy subdirectories
            copy_dir_all(&entry_path, &dest_path)?;
        } else {
            // Copy files
            fs::copy(&entry_path, &dest_path)?;
        }
    }
    Ok(())
}

fn main() {
    let snarkjs_vk_path = std::env::var("IN_VK_PATH").unwrap();
    let snarkjs_public_input_path = std::env::var("IN_PUBLIC_INPUT_PATH").unwrap();
    let snarkjs_proof_path = std::env::var("IN_PROOF_PATH").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let snarkjs_vk_str = read_file_to_string(&snarkjs_vk_path).unwrap();
    let snarkjs_vk: SnarkJsGroth16VerificationKey = serde_json::from_str(&snarkjs_vk_str).unwrap();

    let snarkjs_proof_str = read_file_to_string(&snarkjs_proof_path).unwrap();
    let snarkjs_proof: SnarkJsGroth16Proof = serde_json::from_str(&snarkjs_proof_str).unwrap();

    let snarkjs_public_input_str = read_file_to_string(&snarkjs_public_input_path).unwrap();
    let snarkjs_public_input: SnarkJsGroth16PublicInput = serde_json::from_str(&snarkjs_public_input_str).unwrap();
    copy_dir_all(Path::new("groth16_module_template"), Path::new(out_dir.as_str())).unwrap();

    let output = Command::new("bash")
        .envs(std::env::vars())
        .arg("-c")
        .arg(format!("rsync -a groth16_module_template/ {}/", out_dir))
        .output()
        .unwrap();
    assert!(output.status.success());

    let move_module_path = format!("{out_dir}/sources/groth16.move");
    let template = fs::read_to_string(&move_module_path).unwrap();
    let populated = template
        .replace("__VK_ALPHA_G1__", &Some(&snarkjs_vk.vk_alpha_1).map(as_g1_proj).map(into_aptos_move_bytes_expr).unwrap())
        .replace("__VK_BETA_G2__", &Some(&snarkjs_vk.vk_beta_2).map(as_g2_proj).map(into_aptos_move_bytes_expr).unwrap())
        .replace("__VK_GAMMA_G2__", &Some(&snarkjs_vk.vk_gamma_2).map(as_g2_proj).map(into_aptos_move_bytes_expr).unwrap())
        .replace("__VK_DELTA_G2__", &Some(&snarkjs_vk.vk_delta_2).map(as_g2_proj).map(into_aptos_move_bytes_expr).unwrap())
        .replace("__VK_GAMMA_ABC_G1__", &into_aptos_move_bytes_arr_expr(snarkjs_vk.ic.iter().map(as_g1_proj).collect::<Vec<_>>()))
        .replace("__VK_PUBLIC_INPUTS__", &into_aptos_move_bytes_arr_expr(snarkjs_public_input.iter().map(as_fr).collect::<Vec<_>>()))
        .replace("__PROOF_A__", &Some(&snarkjs_proof.pi_a).map(as_g1_proj).map(into_aptos_move_bytes_expr).unwrap())
        .replace("__PROOF_B__", &Some(&snarkjs_proof.pi_b).map(as_g2_proj).map(into_aptos_move_bytes_expr).unwrap())
        .replace("__PROOF_C__", &Some(&snarkjs_proof.pi_c).map(as_g1_proj).map(into_aptos_move_bytes_expr).unwrap());

    fs::write(&move_module_path, populated).unwrap();
}

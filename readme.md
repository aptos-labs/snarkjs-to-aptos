This tool converts snarkjs outputs (verification keys, public inputs, proofs) into Aptos representations,
which helps you build a Groth16 verifier module for your Aptos contract.

## Supported protocols and curves

Currently, only BN254-based Groth16 is supported. More to be supported in the future.

## How to use

Clone this repo and cd into the repo root.

Assuming you have the following files from snarkjs:
- a verification key at `example/verification_key.json`,
- a public input at `example/public.json`,
- a generated proof at `example/proof.json`.

(An example has been prepared in `example` directory following [snarkjs documentation](https://github.com/iden3/snarkjs)).

Run the following command, and you should see an example Move module generated at `./out`.
```bash
export IN_VK_PATH=example/verification_key.json
export IN_PUBLIC_INPUT_PATH=example/public.json
export IN_PROOF_PATH=example/proof.json
export OUT_DIR=./out
cargo run
```

The example module should contain a test case that successfully verifies your Groth16 proof with your Groth16 verification key, if they match.
```bash
cd out
aptos move test
```

Now you can refactor the test case per your contract's needs.

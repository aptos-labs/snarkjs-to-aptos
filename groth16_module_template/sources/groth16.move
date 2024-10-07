/// Generic implementation of Groth16 (proof verification) as defined in https://eprint.iacr.org/2016/260.pdf, Section 3.2.
/// Actual proof verifiers can be constructed using the pairings supported in the generic algebra module.
/// See the test cases in this module for an example of constructing with BLS12-381 curves.
///
/// **WARNING:** This code has NOT been audited. If using it in a production system, proceed at your own risk.
module groth16_example::groth16 {
    use aptos_std::crypto_algebra::{Element, from_u64, multi_scalar_mul, eq, multi_pairing, upcast, pairing, add, zero};

    /// Proof verification as specified in the original paper,
    /// with the following input (in the original paper notations).
    /// - Verification key: $\left([\alpha]_1, [\beta]_2, [\gamma]_2, [\delta]_2, \left\\{ \left[ \frac{\beta \cdot u_i(x) + \alpha \cdot v_i(x) + w_i(x)}{\gamma} \right]_1 \right\\}\_{i=0}^l \right)$.
    /// - Public inputs: $\\{a_i\\}_{i=1}^l$.
    /// - Proof $\left( \left[ A \right]_1, \left[ B \right]_2, \left[ C \right]_1 \right)$.
    public fun verify_proof<G1,G2,Gt,S>(
        vk_alpha_g1: &Element<G1>,
        vk_beta_g2: &Element<G2>,
        vk_gamma_g2: &Element<G2>,
        vk_delta_g2: &Element<G2>,
        vk_uvw_gamma_g1: &vector<Element<G1>>,
        public_inputs: &vector<Element<S>>,
        proof_a: &Element<G1>,
        proof_b: &Element<G2>,
        proof_c: &Element<G1>,
    ): bool {
        let left = pairing<G1,G2,Gt>(proof_a, proof_b);
        let scalars = vector[from_u64<S>(1)];
        std::vector::append(&mut scalars, *public_inputs);
        let right = zero<Gt>();
        let right = add(&right, &pairing<G1,G2,Gt>(vk_alpha_g1, vk_beta_g2));
        let right = add(&right, &pairing(&multi_scalar_mul(vk_uvw_gamma_g1, &scalars), vk_gamma_g2));
        let right = add(&right, &pairing(proof_c, vk_delta_g2));
        eq(&left, &right)
    }

    #[test_only]
    use aptos_std::crypto_algebra::{deserialize, enable_cryptography_algebra_natives};
    #[test_only]
    use aptos_std::bn254_algebra::{Fr, FormatFrLsb, FormatG1Compr, FormatG2Compr, G1, G2, Gt};
    #[test_only]
    use std::vector;

    #[test(fx = @std)]
    fun test_verify_proof_with_bn254(fx: signer) {
        enable_cryptography_algebra_natives(&fx);

        let vk_alpha_g1 = std::option::extract(&mut deserialize<G1, FormatG1Compr>(&__VK_ALPHA_G1__));
        let vk_beta_g2 = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&__VK_BETA_G2__));
        let vk_gamma_g2 = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&__VK_GAMMA_G2__));
        let vk_delta_g2 = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&__VK_DELTA_G2__));
        let vk_gamma_abc_g1_bytes = __VK_GAMMA_ABC_G1__;
        let public_inputs_bytes = __VK_PUBLIC_INPUTS__;
        assert!(vector::length(&public_inputs_bytes) + 1 == vector::length(&vk_gamma_abc_g1_bytes), 1);

        let vk_gamma_abc_g1 = std::vector::map(vk_gamma_abc_g1_bytes, |item| {
            let bytes: vector<u8> = item;
            std::option::extract(&mut deserialize<G1, FormatG1Compr>(&bytes))
        });

        let public_inputs = std::vector::map(public_inputs_bytes, |item| {
            let bytes: vector<u8> = item;
            std::option::extract(&mut deserialize<Fr, FormatFrLsb>(&bytes))
        });

        let proof_a = std::option::extract(&mut deserialize<G1, FormatG1Compr>(&__PROOF_A__));
        let proof_b = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&__PROOF_B__));
        let proof_c = std::option::extract(&mut deserialize<G1, FormatG1Compr>(&__PROOF_C__));

        assert!(verify_proof<G1, G2, Gt, Fr>(
            &vk_alpha_g1,
            &vk_beta_g2,
            &vk_gamma_g2,
            &vk_delta_g2,
            &vk_gamma_abc_g1,
            &public_inputs,
            &proof_a,
            &proof_b,
            &proof_c,
        ), 1);
    }
}
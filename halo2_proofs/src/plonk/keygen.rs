use halo2_backend::plonk::{
    keygen::{keygen_pk_v2, keygen_vk_v2},
    ProvingKey, VerifyingKey,
};
use halo2_backend::{arithmetic::CurveAffine, poly::commitment::Params};
use halo2_common::plonk::{circuit::Circuit, Error};
use halo2_frontend::circuit::compile_circuit;
use halo2_middleware::ff::FromUniformBytes;

/// Generate a `VerifyingKey` from an instance of `Circuit`.
/// By default, selector compression is turned **off**.
pub fn keygen_vk<'params, C, P, ConcreteCircuit>(
    params: &P,
    circuit: &ConcreteCircuit,
) -> Result<VerifyingKey<C>, Error>
where
    C: CurveAffine,
    P: Params<'params, C>,
    ConcreteCircuit: Circuit<C::Scalar>,
    C::Scalar: FromUniformBytes<64>,
{
    keygen_vk_custom(params, circuit, true)
}

/// Generate a `VerifyingKey` from an instance of `Circuit`.
///
/// The selector compression optimization is turned on only if `compress_selectors` is `true`.
pub fn keygen_vk_custom<'params, C, P, ConcreteCircuit>(
    params: &P,
    circuit: &ConcreteCircuit,
    compress_selectors: bool,
) -> Result<VerifyingKey<C>, Error>
where
    C: CurveAffine,
    P: Params<'params, C>,
    ConcreteCircuit: Circuit<C::Scalar>,
    C::Scalar: FromUniformBytes<64>,
{
    let (compiled_circuit, _, _) = compile_circuit(params.k(), circuit, compress_selectors)?;
    let mut vk = keygen_vk_v2(params, &compiled_circuit)?;
    vk.compress_selectors = compress_selectors;
    Ok(vk)
}

/// Generate a `ProvingKey` from a `VerifyingKey` and an instance of `Circuit`.
pub fn keygen_pk<'params, C, P, ConcreteCircuit>(
    params: &P,
    vk: VerifyingKey<C>,
    circuit: &ConcreteCircuit,
) -> Result<ProvingKey<C>, Error>
where
    C: CurveAffine,
    P: Params<'params, C>,
    ConcreteCircuit: Circuit<C::Scalar>,
{
    let (compiled_circuit, _, _) = compile_circuit(params.k(), circuit, vk.compress_selectors)?;
    keygen_pk_v2(params, vk, &compiled_circuit)
}

#[cfg(test)]
mod test {
    use crate::arithmetic::{eval_polynomial, FieldExt};
    use crate::plonk::Error;
    use crate::poly::commitment::ParamsProver;
    use crate::poly::commitment::{Blind, ParamsVerifier, MSM};
    use crate::poly::query::PolynomialPointer;
    use crate::poly::{
        commitment::{CommitmentScheme, Params, Prover, Verifier},
        query::{ProverQuery, VerifierQuery},
        strategy::VerificationStrategy,
        EvaluationDomain,
    };
    use crate::poly::{Coeff, Polynomial};
    use crate::transcript::{
        self, Blake2bRead, Blake2bWrite, Challenge255, EncodedChallenge, TranscriptRead,
        TranscriptReadBuffer, TranscriptWrite, TranscriptWriterBuffer,
    };
    use ff::Field;
    use group::{Curve, Group};
    use halo2curves::CurveAffine;
    use rand_core::{OsRng, RngCore};
    use std::io::{Read, Write};

    #[test]
    fn test_roundtrip_ipa() {
        use crate::poly::ipa::commitment::{IPACommitmentScheme, ParamsIPA};
        use crate::poly::ipa::multiopen::{ProverIPA, VerifierIPA};
        use crate::poly::ipa::strategy::AccumulatorStrategy;
        use halo2curves::pasta::{Ep, EqAffine, Fp};

        const K: u32 = 4;

        let params = ParamsIPA::<EqAffine>::new(K);

        let proof = create_proof::<
            IPACommitmentScheme<EqAffine>,
            ProverIPA<_>,
            _,
            Blake2bWrite<_, _, Challenge255<_>>,
        >(&params);

        let verifier_params = params.verifier_params();

        verify::<
            IPACommitmentScheme<EqAffine>,
            VerifierIPA<_>,
            _,
            Blake2bRead<_, _, Challenge255<_>>,
            AccumulatorStrategy<_>,
        >(verifier_params, &proof[..], false);

        verify::<
            IPACommitmentScheme<EqAffine>,
            VerifierIPA<_>,
            _,
            Blake2bRead<_, _, Challenge255<_>>,
            AccumulatorStrategy<_>,
        >(verifier_params, &proof[..], true);
    }

    #[test]
    fn test_roundtrip_gwc() {
        use crate::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
        use crate::poly::kzg::multiopen::{ProverGWC, VerifierGWC};
        use crate::poly::kzg::strategy::AccumulatorStrategy;
        use halo2curves::bn256::{Bn256, G1Affine};
        use halo2curves::pairing::Engine;

        const K: u32 = 4;

        let params = ParamsKZG::<Bn256>::new(K);

        let proof =
            create_proof::<_, ProverGWC<_>, _, Blake2bWrite<_, _, Challenge255<_>>>(&params);

        let verifier_params = params.verifier_params();

        verify::<_, VerifierGWC<_>, _, Blake2bRead<_, _, Challenge255<_>>, AccumulatorStrategy<_>>(
            verifier_params,
            &proof[..],
            false,
        );

        verify::<
            KZGCommitmentScheme<Bn256>,
            VerifierGWC<_>,
            _,
            Blake2bRead<_, _, Challenge255<_>>,
            AccumulatorStrategy<_>,
        >(verifier_params, &proof[..], true);
    }

    #[test]
    fn test_roundtrip_shplonk() {
        use crate::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
        use crate::poly::kzg::multiopen::{ProverSHPLONK, VerifierSHPLONK};
        use crate::poly::kzg::strategy::AccumulatorStrategy;
        use halo2curves::bn256::{Bn256, G1Affine};
        use halo2curves::pairing::Engine;

        const K: u32 = 4;

        let params = ParamsKZG::<Bn256>::new(K);

        let proof = create_proof::<
            KZGCommitmentScheme<Bn256>,
            ProverSHPLONK<_>,
            _,
            Blake2bWrite<_, _, Challenge255<_>>,
        >(&params);

        let verifier_params = params.verifier_params();

        verify::<
            KZGCommitmentScheme<Bn256>,
            VerifierSHPLONK<_>,
            _,
            Blake2bRead<_, _, Challenge255<_>>,
            AccumulatorStrategy<_>,
        >(verifier_params, &proof[..], false);

        verify::<
            KZGCommitmentScheme<Bn256>,
            VerifierSHPLONK<_>,
            _,
            Blake2bRead<_, _, Challenge255<_>>,
            AccumulatorStrategy<_>,
        >(verifier_params, &proof[..], true);
    }

    fn verify<
        'a,
        'params,
        Scheme: CommitmentScheme,
        V: Verifier<'params, Scheme>,
        E: EncodedChallenge<Scheme::Curve>,
        T: TranscriptReadBuffer<&'a [u8], Scheme::Curve, E>,
        Strategy: VerificationStrategy<'params, Scheme, V, Output = Strategy>,
    >(
        params: &'params Scheme::ParamsVerifier,
        proof: &'a [u8],
        should_fail: bool,
    ) {
        let verifier = V::new(params);

        let mut transcript = T::init(proof);

        let a = transcript.read_point().unwrap();
        let b = transcript.read_point().unwrap();
        let c = transcript.read_point().unwrap();

        let x = transcript.squeeze_challenge();
        let y = transcript.squeeze_challenge();

        let avx = transcript.read_scalar().unwrap();
        let bvx = transcript.read_scalar().unwrap();
        let cvy = transcript.read_scalar().unwrap();

        let valid_queries = std::iter::empty()
            .chain(Some(VerifierQuery::new_commitment(&a, x.get_scalar(), avx)))
            .chain(Some(VerifierQuery::new_commitment(&b, x.get_scalar(), bvx)))
            .chain(Some(VerifierQuery::new_commitment(&c, y.get_scalar(), cvy)));

        let invalid_queries = std::iter::empty()
            .chain(Some(VerifierQuery::new_commitment(&a, x.get_scalar(), avx)))
            .chain(Some(VerifierQuery::new_commitment(&b, x.get_scalar(), avx)))
            .chain(Some(VerifierQuery::new_commitment(&c, y.get_scalar(), cvy)));

        let queries = if should_fail {
            invalid_queries.clone()
        } else {
            valid_queries.clone()
        };

        {
            let strategy = Strategy::new(params);
            let strategy = strategy
                .process(|msm_accumulator| {
                    verifier
                        .verify_proof(&mut transcript, queries.clone(), msm_accumulator)
                        .map_err(|_| Error::Opening)
                })
                .unwrap();

            assert_eq!(strategy.finalize(), !should_fail);
        }
    }

    fn create_proof<
        'params,
        Scheme: CommitmentScheme,
        P: Prover<'params, Scheme>,
        E: EncodedChallenge<Scheme::Curve>,
        T: TranscriptWriterBuffer<Vec<u8>, Scheme::Curve, E>,
    >(
        params: &'params Scheme::ParamsProver,
    ) -> Vec<u8> {
        let domain = EvaluationDomain::new(1, params.k());

        let mut ax = domain.empty_coeff();
        for (i, a) in ax.iter_mut().enumerate() {
            *a = <<Scheme as CommitmentScheme>::Curve as CurveAffine>::ScalarExt::from(
                10 + i as u64,
            );
        }

        let mut bx = domain.empty_coeff();
        for (i, a) in bx.iter_mut().enumerate() {
            *a = <<Scheme as CommitmentScheme>::Curve as CurveAffine>::ScalarExt::from(
                100 + i as u64,
            );
        }

        let mut cx = domain.empty_coeff();
        for (i, a) in cx.iter_mut().enumerate() {
            *a = <<Scheme as CommitmentScheme>::Curve as CurveAffine>::ScalarExt::from(
                100 + i as u64,
            );
        }

        let mut transcript = T::init(vec![]);

        let blind = Blind::new(&mut OsRng);
        let a = params.commit(&ax, blind).to_affine();
        let b = params.commit(&bx, blind).to_affine();
        let c = params.commit(&cx, blind).to_affine();

        transcript.write_point(a).unwrap();
        transcript.write_point(b).unwrap();
        transcript.write_point(c).unwrap();

        let x = transcript.squeeze_challenge();
        let y = transcript.squeeze_challenge();

        let avx = eval_polynomial(&ax, x.get_scalar());
        let bvx = eval_polynomial(&bx, x.get_scalar());
        let cvy = eval_polynomial(&cx, y.get_scalar());

        transcript.write_scalar(avx).unwrap();
        transcript.write_scalar(bvx).unwrap();
        transcript.write_scalar(cvy).unwrap();

        let queries = [
            ProverQuery {
                point: x.get_scalar(),
                poly: &ax,
                blind,
            },
            ProverQuery {
                point: x.get_scalar(),
                poly: &bx,
                blind,
            },
            ProverQuery {
                point: y.get_scalar(),
                poly: &cx,
                blind,
            },
        ]
        .to_vec();

        let prover = P::new(params);
        prover
            .create_proof(&mut OsRng, &mut transcript, queries)
            .unwrap();

        transcript.finalize()
    }
}

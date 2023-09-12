# secpq_curves

A merging of https://github.com/privacy-scaling-explorations/halo2curves and https://github.com/zcash/pasta_curves to implement the Secp/Secq cycle of curves. There is also a poorly documented SAGE script that walks through how certain constants were picked, like the isogeny map for Secq for its Simplied SWU mappings. This code was necessary for hash_to_curve for both Secp and Secq, as random points are necessary in the IPA param generation.

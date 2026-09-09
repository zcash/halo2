import sage.schemes.elliptic_curves.isogeny_small_degree as isd

p = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f
print(p)
Fp = GF(p)
q = 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141
print(q)
Fq = GF(q)

pA = Fp(0x0000000000000000000000000000000000000000000000000000000000000000)
pB = Fp(0x0000000000000000000000000000000000000000000000000000000000000007)
Ep = EllipticCurve(Fp, (pA, pB))
Gp = Ep(0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798, 0x483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8)
print("Order of Ep is q:", Ep.order() == q)

qA = Fq(0x0000000000000000000000000000000000000000000000000000000000000000)
qB = Fq(0x0000000000000000000000000000000000000000000000000000000000000007)
Eq = EllipticCurve(Fq, (qA, qB))
Gq = Eq(0x9214b8774eb1412be7590cbef17c26fc2cabb9347a25101b860fee175831bb20, 0x28cb5b51a30b5532ecc5b53440a7598a836a6e341a88892a14a1bc519466eb6b)
print("Order of Eq is p:", Eq.order() == p)
      
def find_iso(E):
    for p_test in primes(30):
        isos = [ i for i in isd.isogenies_prime_degree(E, p_test) if i.codomain().j_invariant() not in (0, 1728) ]
        if len(isos) > 0:
            return isos[0].dual()
    return None
    
IsoEpA = 0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533
IsoEpB = 1771
IsoEp = EllipticCurve(GF(p), [IsoEpA, IsoEpB])
isogeny_ep = EllipticCurveIsogeny(E=Ep, kernel=None, codomain=IsoEp, degree=3).dual()
print(isogeny_ep.rational_maps())
isogeny_eq = find_iso(Eq)
print([k for k in isogeny_eq.rational_maps()])

def find_z_sswu(F, A, B):
    R.<xx> = F[]                       # Polynomial ring over F
    g = xx^3 + F(A) * xx + F(B)        # y^2 = g(x) = x^3 + A * x + B
    ctr = F.gen()
    while True:
        for Z_cand in (F(ctr), F(-ctr)):
            # Criterion 1: Z is non-square in F.
            if is_square(Z_cand):
                continue
            # Criterion 2: Z != -1 in F.
            if Z_cand == F(-1):
                continue
            # Criterion 3: g(x) - Z is irreducible over F.
            if not (g - Z_cand).is_irreducible():
                continue
            # Criterion 4: g(B / (Z * A)) is square in F.
            if is_square(g(B / (Z_cand * A))):
                return Z_cand
        ctr += 1

print("Z for IsoEp", find_z_sswu(Fp, IsoEpA, IsoEpB), "=", Integer(find_z_sswu(Fp, IsoEpA, IsoEpB)) - Integer(p))
print("Z for IsoEq", find_z_sswu(Fq, isogeny_eq.domain().a4(), isogeny_eq.domain().a6()), "=", Integer(find_z_sswu(Fq, isogeny_eq.domain().a4(), isogeny_eq.domain().a6())) - Integer(q))
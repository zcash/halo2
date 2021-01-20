# Nullifiers

The nullifier design we use for Orchard is

$$\mathsf{nf} = [F_{\mathsf{nk}}(\rho) + \psi \pmod{p}] \mathcal{G} + \mathsf{cm},$$

where:
- $F$ is a keyed circuit-efficient PRF (such as Rescue).
- $\rho$ is unique to this output. As with $\mathsf{h_{Sig}}$ in Sprout, $\rho$ includes
  the nullifiers of any Orchard notes being spent in the same action. Given that an action
  consists of a single spend and a single output, we set $\rho$ to be the nullifier of the
  spent note.
- $\psi$ is sender-controlled randomness. It is not required to be unique, and in practice
  is derived from both $\rho$ and a sender-selected random value $\mathsf{rseed}$:
  $$\psi = KDF^\psi(\rho, \mathsf{rseed}).$$
- $\mathcal{G}$ is a fixed independent base.

This gives a note structure of

$$(addr, v, \rho, \psi, \mathsf{rcm}).$$

The note plaintext includes $\mathsf{rseed}$ in place of $\psi$ and $\mathsf{rcm}$, and
omits $\rho$ (which is a public part of the action).

## Security properties

We care about several security properties for our nullifiers:

- **Balance:** can I forge money?

- **Note Privacy:** can I gain information about notes only from the public block chain?
  - This describes notes sent in-band.

- **Note Privacy (OOB):** can I gain information about notes sent out-of-band, only from
  the public block chain?
  - In this case, we assume privacy of the channel over which the note is sent, and that
    the adversary does not have access to any notes sent to the same address which are
    then spent (so that the nullifier is on the block chain somewhere).

- **Spend Unlinkability:** given the incoming viewing key for an address, and not the full
  viewing key, can I (possibly the sender) detect spends of any notes sent to that address?
  - We're giving $ivk$ to the attacker and allowing it to be the sender in order to make
    this property as strong as possible: they will have *all* the notes sent to that
    address.

- **Faerie Resistance:** can I perform a Faerie Gold attack (i.e. cause notes to be
  accepted that are unspendable)?
  - We're giving the full viewing key to the attacker and allowing it to be the sender in
    order to make this property as strong as possible: they will have *all* the notes sent
    to that address, and be able to derive *every* nullifier.

We assume (and instantiate elsewhere) the following primitives:

- $GH$ is a cryptographic hash into the group (such as BLAKE2s with simplified SWU), used
  to derive all fixed independent bases.
- $E$ is an elliptic curve (such as Pallas).
- $KDF$ is the note encryption key derivation function.

For our chosen design, our desired security properties rely on the following assumptions:

$$
\begin{array}{|l|l|}
\text{Balance} & DL_E \\
\text{Note Privacy} & HashDH^{KDF}_E \\
\text{Note Privacy (OOB)} & \text{Near perfect} \ddagger \\
\text{Spend Unlinkability} & DDH_E^\dagger \vee PRF_F \\
\text{Faerie Resistance} & DL_E \\
\end{array}
$$

$HashDH^{KDF}_E$ is computational Diffie-Hellman using $KDF$ for the key derivation, with
one-time ephemeral keys. This assumption is heuristically weaker than $DDH_E$ but stronger
than $DL_E$.

We omit $RO_{GH}$ as a security assumption because we only rely on the random oracle
applied to fixed inputs defined by the protocol, i.e. to generate the fixed base
$\mathcal{G}$, not to attacker-specified inputs.

> $\dagger$ We additionally assume that for any input $x$,
> $\{F_{\mathsf{nk}}(x) : \mathsf{nk} \in E\}$ gives a scalar in an adequate range for
> $DDH_E$. (Otherwise, $F$ could be trivial, e.g. independent of $\mathsf{nk}$.)
>
> $\ddagger$ Statistical distance $< 2^{-167.8}$ from perfect.

## Considered alternatives

$\color{red}{\textsf{⚠ Caution}}$: be skeptical of the claims in this table about what
problem(s) each security property depends on. They may not be accurate and are definitely
not fully rigorous.

$$
\begin{array}{|c|l|c|c|c|c|c|}
\hline
\mathsf{nf} & Note & \text{Balance} & \text{Note Privacy} & \text{Note Privacy (OOB)} & \text{Spend Unlinkability} & \text{Faerie Resistance} & \text{Reason not to use} \\\hline
[\mathsf{nk}] [\theta] H & (addr, v, H, \theta, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E & RO_{GH} \wedge DL_E & \text{No SU for DL-breaking} \\\hline
[\mathsf{nk}] H + [\mathsf{rnf}] \mathcal{I} & (addr, v, H, \mathsf{rnf}, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E & RO_{GH} \wedge DL_E & \text{No SU for DL-breaking} \\\hline
Hash([\mathsf{nk}] [\theta] H) & (addr, v, H, \theta, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E \vee Pre_{Hash} & Coll_{Hash} \wedge RO_{GH} \wedge DL_E & Coll_{Hash} \text{ for FR} \\\hline
Hash([\mathsf{nk}] H + [\mathsf{rnf}] \mathcal{I}) & (addr, v, H, \mathsf{rnf}, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E \vee Pre_{Hash} & Coll_{Hash} \wedge RO_{GH} \wedge DL_E & Coll_{Hash} \text{ for FR} \\\hline
[F_{\mathsf{nk}}(\psi)] [\theta] H & (addr, v, H, \theta, \psi, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E^\dagger \vee PRF_F & RO_{GH} \wedge DL_E & \text{Performance (2 variable-base)} \\\hline
[F_{\mathsf{nk}}(\psi)] H + [\mathsf{rnf}] \mathcal{I} & (addr, v, H, \mathsf{rnf}, \psi, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E^\dagger \vee PRF_F & RO_{GH} \wedge DL_E & \text{Performance (1 variable- + 1 fixed-base)} \\\hline
[F_{\mathsf{nk}}(\psi)] \mathcal{G} + [\theta] H & (addr, v, H, \theta, \psi, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E^\dagger \vee PRF_F & RO_{GH} \wedge DL_E & \text{Performance (1 variable- + 1 fixed-base)} \\\hline
[F_{\mathsf{nk}}(\psi)] H + \mathsf{cm} & (addr, v, H, \psi, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & DDH_E^\dagger & DDH_E^\dagger \vee PRF_F & RO_{GH} \wedge DL_E & \text{NP(OOB) not perfect} \\\hline
[F_{\mathsf{nk}}(\rho, \psi)] \mathcal{G} + \mathsf{cm} & (addr, v, \rho, \psi, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & DDH_E^\dagger  & DDH_E^\dagger \vee PRF_F & DL_E & \text{NP(OOB) not perfect} \\\hline
[F_{\mathsf{nk}}(\rho)] \mathcal{G} + \mathsf{cm} & (addr, v, \rho, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & DDH_E^\dagger & DDH_E^\dagger \vee PRF_F & DL_E & \text{NP(OOB) not perfect} \\\hline
[F_{\mathsf{nk}}(\rho, \psi)] \mathcal{G_v} + [\mathsf{rnf}] \mathcal{I} & (addr, v, \rho, \mathsf{rnf}, \psi, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E^\dagger \vee PRF_F & Coll_F \wedge DL_E & Coll_F \text{ for FR} \\\hline
[F_{\mathsf{nk}}(\rho)] \mathcal{G_v} + [\mathsf{rnf}] \mathcal{I} & (addr, v, \rho, \mathsf{rnf}, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E^\dagger \vee PRF_F & Coll_F \wedge DL_E & Coll_F \text{ for FR} \\\hline
[F_{\mathsf{nk}}(\rho) + \psi \pmod{p}] \mathcal{G_v} & (addr, v, \rho, \psi, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Near perfect} \ddagger & DDH_E^\dagger \vee PRF_F & \color{red}{\text{broken}} & \text{broken for FR} \\\hline
[F_{\mathsf{nk}}(\rho, \psi)] \mathcal{G} + Commit^{\mathsf{nf}}_{\mathsf{rnf}}(v, \rho) & (addr, v, \rho, \mathsf{rnf}, \psi, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E^\dagger \vee PRF_F & DL_E & \text{Performance (2 fixed-base)} \\\hline
[F_{\mathsf{nk}}(\rho)] \mathcal{G} + Commit^{\mathsf{nf}}_{\mathsf{rnf}}(v, \rho) & (addr, v, \rho, \mathsf{rnf}, \mathsf{rcm}) & DL_E & HashDH^{KDF}_E & \text{Perfect} & DDH_E^\dagger \vee PRF_F & DL_E & \text{Performance (2 fixed-base)} \\\hline
\end{array}
$$

In the above alternatives:
- $Hash$ is a keyed circuit-efficient hash (such as Rescue).
- $\mathcal{I}$ is an fixed independent base, independent of $\mathcal{G}$ and any others
  returned by $GH$.
- $\mathcal{G_v}$ is a pair of fixed independent bases (independent of all others), where
  the specific choice of base depends on whether the note has zero value.
- $H$ is a base unique to this output.
  - For non-zero-valued notes, $H = GH(\rho)$. As with $\mathsf{h_{Sig}}$ in Sprout,
    $\rho$ includes the nullifiers of any Orchard notes being spent in the same action.
  - For zero-valued notes, $H$ is constrained by the circuit to a fixed base independent
    of $\mathcal{I}$ and any others returned by $GH$.

The $Commit^{\mathsf{nf}}$ variants enabled nullifier domain separation based on note
value, without directly depending on $\mathsf{cm}$ (which in its native type is a base
field element, not a group element). We decided instead to follow Sapling by defining an
intermediate representation of $\mathsf{cm}$ as a group element, that is only used in
nullifier computation.

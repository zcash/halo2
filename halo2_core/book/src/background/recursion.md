## Recursion
> Alternative terms: Induction; Accumulation scheme; Proof-carrying data

However, the computation of $G$ requires a length-$2^k$ multiexponentiation
$\langle \mathbf{G}, \mathbf{s}\rangle,$ where $\mathbf{s}$ is composed of the round
challenges $u_1, \cdots, u_k$ arranged in a binary counting structure. This is the
linear-time computation that we want to amortise across a batch of proof instances.
Instead of computing $G,$ notice that we can express $G$ as a commitment to a polynomial

$$G = \text{Commit}(\sigma, g(X, u_1, \cdots, u_k)),$$

where $g(X, u_1, \cdots, u_k) := \prod_{i=1}^k (u_i + u_i^{-1}X^{2^{i-1}})$ is a
polynomial with degree $2^k - 1.$ 
 
|  |  | 
| -------- | -------- | 
| <img src="https://i.imgur.com/vMXKFDV.png" width=1900> | Since $G$ is a commitment, it can be checked in an inner product argument. The verifier circuit witnesses $G$ and brings $G, u_1, \cdots, u_k$ out as public inputs to the proof $\pi.$ The next verifier instance checks $\pi$ using the inner product argument; this includes checking that $G = \text{Commit}(g(X, u_1, \cdots, u_k))$ evaluates at some random point to the expected value for the given challenges $u_1, \cdots, u_k.$ Recall from the [previous section](#Polynomial-commitment-using-inner-product-argument) that this check only requires $\log d$ work. <br><br> At the end of checking $\pi$ and $G,$ the circuit is left with a new $G',$ along with the $u_1', \cdots, u_k'$ challenges sampled for the check. To fully accept $\pi$ as valid, we should perform a linear-time computation of $G' = \langle\mathbf{G}, \mathbf{s}'\rangle$. Once again, we delay this computation by witnessing $G'$ and bringing $G, u_1, \cdots, u_k$ out as public inputs to the proof $\pi.$ <br><br> This goes on from one proof instance to the next, until we are satisfied with the size of our batch of proofs. We finally perform a single linear-time computation, thus deciding the validity of the whole batch.   |

We recall from the section [Cycles of curves](curves.md#cycles-of-curves) that we can
instantiate this protocol over a two-cycle, where a proof produced by one curve is
efficiently verified in the circuit of the other curve. However, some of these verifier
checks can actually be efficiently performed in the native circuit; these are "deferred"
to the next native circuit (see diagram below) instead of being immediately passed over to
the other curve. 

![](https://i.imgur.com/l4HrYgE.png)

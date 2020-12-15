# Actions

In Sprout, we had a single proof that represented two spent notes and two new notes. This
was necessary in order to faciliate spending multiple notes in a single transaction (to
balance value, an output of one JoinSplit could be spent in the next one), but also
provided a minimal level of arity-hiding: single-JoinSplit transactions all looked like
2-in 2-out transactions, and in multi-JoinSplit transactions each JoinSplit looked like a
1-in 1-out.

In Sapling, we switched to using value commitments to balance the transaction, removing
the min-2 arity requirement. We opted for one proof per spent note and one (much simpler)
proof per output note, which greatly improved the performance of generating outputs, but
removed any arity-hiding from the proofs (instead having the transaction builder pad
transactions to 1-in, 2-out).

For Orchard, we take a combined approach: we define an Orchard transaction as containing a
bundle of actions, where each action is both a spend and an output. This provides the same
inherent arity-hiding as multi-JoinSplit Sprout, but using Sapling value commitments to
balance the transaction without doubling its size.

TODO: Depending on the circuit cost, we _may_ switch to having an action internally
represent either a spend or an output. Externally spends and outputs would still be
indistinguishable, but the transaction would be larger.

## Memo fields

TODO: One memo per tx vs one memo per output

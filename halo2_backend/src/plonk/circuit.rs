use group::ff::Field;
use halo2_middleware::circuit::{Any, ChallengeMid, ColumnMid, Gate};
use halo2_middleware::expression::{Expression, Variable};
use halo2_middleware::poly::Rotation;
use halo2_middleware::{lookup, permutation::ArgumentMid, shuffle};

// TODO: Reuse ColumnMid inside this.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryBack {
    /// Query index
    pub(crate) index: usize,
    /// Column index
    pub(crate) column_index: usize,
    /// The type of the column.
    pub(crate) column_type: Any,
    /// Rotation of this query
    pub(crate) rotation: Rotation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VarBack {
    /// This is a generic column query
    Query(QueryBack),
    /// This is a challenge
    Challenge(ChallengeMid),
}

impl Variable for VarBack {
    fn degree(&self) -> usize {
        match self {
            VarBack::Query(_) => 1,
            VarBack::Challenge(_) => 0,
        }
    }

    fn complexity(&self) -> usize {
        match self {
            VarBack::Query(_) => 1,
            VarBack::Challenge(_) => 0,
        }
    }

    fn write_identifier<W: std::io::Write>(&self, _writer: &mut W) -> std::io::Result<()> {
        unimplemented!("unused method")
    }
}

pub(crate) type ExpressionBack<F> = Expression<F, VarBack>;
pub(crate) type GateBack<F> = Gate<F, VarBack>;
pub(crate) type LookupArgumentBack<F> = lookup::Argument<F, VarBack>;
pub(crate) type ShuffleArgumentBack<F> = shuffle::Argument<F, VarBack>;
pub(crate) type PermutationArgumentBack = ArgumentMid;

/// This is a description of the circuit environment, such as the gate, column and permutation
/// arrangements.  This type is internal to the backend and will appear in the verifying key.
#[derive(Debug, Clone)]
pub struct ConstraintSystemBack<F: Field> {
    pub(crate) num_fixed_columns: usize,
    pub(crate) num_advice_columns: usize,
    pub(crate) num_instance_columns: usize,
    pub(crate) num_challenges: usize,

    /// Contains the index of each advice column that is left unblinded.
    pub(crate) unblinded_advice_columns: Vec<usize>,

    /// Contains the phase for each advice column. Should have same length as num_advice_columns.
    pub(crate) advice_column_phase: Vec<u8>,
    /// Contains the phase for each challenge. Should have same length as num_challenges.
    pub(crate) challenge_phase: Vec<u8>,

    pub(crate) gates: Vec<GateBack<F>>,
    pub(crate) advice_queries: Vec<(ColumnMid, Rotation)>,
    // Contains an integer for each advice column
    // identifying how many distinct queries it has
    // so far; should be same length as num_advice_columns.
    pub(crate) num_advice_queries: Vec<usize>,
    pub(crate) instance_queries: Vec<(ColumnMid, Rotation)>,
    pub(crate) fixed_queries: Vec<(ColumnMid, Rotation)>,

    // Permutation argument for performing equality constraints
    pub(crate) permutation: PermutationArgumentBack,

    // Vector of lookup arguments, where each corresponds to a sequence of
    // input expressions and a sequence of table expressions involved in the lookup.
    pub(crate) lookups: Vec<LookupArgumentBack<F>>,

    // Vector of shuffle arguments, where each corresponds to a sequence of
    // input expressions and a sequence of shuffle expressions involved in the shuffle.
    pub(crate) shuffles: Vec<ShuffleArgumentBack<F>>,

    // The minimum degree required by the circuit, which can be set to a
    // larger amount than actually needed. This can be used, for example, to
    // force the permutation argument to involve more columns in the same set.
    pub(crate) minimum_degree: Option<usize>,
}

impl<F: Field> ConstraintSystemBack<F> {
    /// Compute the degree of the constraint system (the maximum degree of all
    /// constraints).
    pub fn degree(&self) -> usize {
        // The permutation argument will serve alongside the gates, so must be
        // accounted for.
        let mut degree = permutation_argument_required_degree();

        // The lookup argument also serves alongside the gates and must be accounted
        // for.
        degree = std::cmp::max(
            degree,
            self.lookups
                .iter()
                .map(|l| lookup_argument_required_degree(l))
                .max()
                .unwrap_or(1),
        );

        // The lookup argument also serves alongside the gates and must be accounted
        // for.
        degree = std::cmp::max(
            degree,
            self.shuffles
                .iter()
                .map(|l| shuffle_argument_required_degree(l))
                .max()
                .unwrap_or(1),
        );

        // Account for each gate to ensure our quotient polynomial is the
        // correct degree and that our extended domain is the right size.
        degree = std::cmp::max(
            degree,
            self.gates
                .iter()
                .map(|gate| gate.poly.degree())
                .max()
                .unwrap_or(0),
        );

        std::cmp::max(degree, self.minimum_degree.unwrap_or(1))
    }

    /// Compute the number of blinding factors necessary to perfectly blind
    /// each of the prover's witness polynomials.
    pub fn blinding_factors(&self) -> usize {
        // All of the prover's advice columns are evaluated at no more than
        let factors = *self.num_advice_queries.iter().max().unwrap_or(&1);
        // distinct points during gate checks.

        // - The permutation argument witness polynomials are evaluated at most 3 times.
        // - Each lookup argument has independent witness polynomials, and they are
        //   evaluated at most 2 times.
        let factors = std::cmp::max(3, factors);

        // Each polynomial is evaluated at most an additional time during
        // multiopen (at x_3 to produce q_evals):
        let factors = factors + 1;

        // h(x) is derived by the other evaluations so it does not reveal
        // anything; in fact it does not even appear in the proof.

        // h(x_3) is also not revealed; the verifier only learns a single
        // evaluation of a polynomial in x_1 which has h(x_3) and another random
        // polynomial evaluated at x_3 as coefficients -- this random polynomial
        // is "random_poly" in the vanishing argument.

        // Add an additional blinding factor as a slight defense against
        // off-by-one errors.
        factors + 1
    }

    /// Returns the minimum necessary rows that need to exist in order to
    /// account for e.g. blinding factors.
    pub fn minimum_rows(&self) -> usize {
        self.blinding_factors() // m blinding factors
            + 1 // for l_{-(m + 1)} (l_last)
            + 1 // for l_0 (just for extra breathing room for the permutation
                // argument, to essentially force a separation in the
                // permutation polynomial between the roles of l_last, l_0
                // and the interstitial values.)
            + 1 // for at least one row
    }

    pub fn get_any_query_index(&self, column: ColumnMid, at: Rotation) -> usize {
        let queries = match column.column_type {
            Any::Advice => &self.advice_queries,
            Any::Fixed => &self.fixed_queries,
            Any::Instance => &self.instance_queries,
        };
        for (index, instance_query) in queries.iter().enumerate() {
            if instance_query == &(column, at) {
                return index;
            }
        }
        panic!("get_any_query_index called for non-existent query");
    }

    /// Returns the list of phases
    pub fn phases(&self) -> impl Iterator<Item = u8> {
        let max_phase = self
            .advice_column_phase
            .iter()
            .max()
            .copied()
            .unwrap_or_default();
        0..=max_phase
    }

    /// Obtain a pinned version of this constraint system; a structure with the
    /// minimal parameters needed to determine the rest of the constraint
    /// system.
    pub fn pinned(&self) -> PinnedConstraintSystem<'_, F> {
        PinnedConstraintSystem {
            num_fixed_columns: &self.num_fixed_columns,
            num_advice_columns: &self.num_advice_columns,
            num_instance_columns: &self.num_instance_columns,
            num_challenges: &self.num_challenges,
            advice_column_phase: &self.advice_column_phase,
            challenge_phase: &self.challenge_phase,
            gates: PinnedGates(&self.gates),
            fixed_queries: &self.fixed_queries,
            advice_queries: &self.advice_queries,
            instance_queries: &self.instance_queries,
            permutation: &self.permutation,
            lookups: &self.lookups,
            shuffles: &self.shuffles,
            minimum_degree: &self.minimum_degree,
        }
    }
}

struct PinnedGates<'a, F: Field>(&'a Vec<GateBack<F>>);

impl<'a, F: Field> std::fmt::Debug for PinnedGates<'a, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_list()
            .entries(self.0.iter().map(|gate| &gate.poly))
            .finish()
    }
}

/// Represents the minimal parameters that determine a `ConstraintSystem`.
pub struct PinnedConstraintSystem<'a, F: Field> {
    num_fixed_columns: &'a usize,
    num_advice_columns: &'a usize,
    num_instance_columns: &'a usize,
    num_challenges: &'a usize,
    advice_column_phase: &'a Vec<u8>,
    challenge_phase: &'a Vec<u8>,
    gates: PinnedGates<'a, F>,
    advice_queries: &'a Vec<(ColumnMid, Rotation)>,
    instance_queries: &'a Vec<(ColumnMid, Rotation)>,
    fixed_queries: &'a Vec<(ColumnMid, Rotation)>,
    permutation: &'a PermutationArgumentBack,
    lookups: &'a Vec<LookupArgumentBack<F>>,
    shuffles: &'a Vec<ShuffleArgumentBack<F>>,
    minimum_degree: &'a Option<usize>,
}

impl<'a, F: Field> std::fmt::Debug for PinnedConstraintSystem<'a, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("PinnedConstraintSystem");
        debug_struct
            .field("num_fixed_columns", self.num_fixed_columns)
            .field("num_advice_columns", self.num_advice_columns)
            .field("num_instance_columns", self.num_instance_columns);
        // Only show multi-phase related fields if it's used.
        if *self.num_challenges > 0 {
            debug_struct
                .field("num_challenges", self.num_challenges)
                .field("advice_column_phase", self.advice_column_phase)
                .field("challenge_phase", self.challenge_phase);
        }
        debug_struct
            .field("gates", &self.gates)
            .field("advice_queries", self.advice_queries)
            .field("instance_queries", self.instance_queries)
            .field("fixed_queries", self.fixed_queries)
            .field("permutation", self.permutation)
            .field("lookups", self.lookups);
        if !self.shuffles.is_empty() {
            debug_struct.field("shuffles", self.shuffles);
        }
        debug_struct.field("minimum_degree", self.minimum_degree);
        debug_struct.finish()
    }
}

// Cost functions: arguments required degree

/// Returns the minimum circuit degree required by the permutation argument.
/// The argument may use larger degree gates depending on the actual
/// circuit's degree and how many columns are involved in the permutation.
fn permutation_argument_required_degree() -> usize {
    // degree 2:
    // l_0(X) * (1 - z(X)) = 0
    //
    // We will fit as many polynomials p_i(X) as possible
    // into the required degree of the circuit, so the
    // following will not affect the required degree of
    // this middleware.
    //
    // (1 - (l_last(X) + l_blind(X))) * (
    //   z(\omega X) \prod (p(X) + \beta s_i(X) + \gamma)
    // - z(X) \prod (p(X) + \delta^i \beta X + \gamma)
    // )
    //
    // On the first sets of columns, except the first
    // set, we will do
    //
    // l_0(X) * (z(X) - z'(\omega^(last) X)) = 0
    //
    // where z'(X) is the permutation for the previous set
    // of columns.
    //
    // On the final set of columns, we will do
    //
    // degree 3:
    // l_last(X) * (z'(X)^2 - z'(X)) = 0
    //
    // which will allow the last value to be zero to
    // ensure the argument is perfectly complete.

    // There are constraints of degree 3 regardless of the
    // number of columns involved.
    3
}

fn lookup_argument_required_degree<F: Field, V: Variable>(arg: &lookup::Argument<F, V>) -> usize {
    assert_eq!(arg.input_expressions.len(), arg.table_expressions.len());

    // The first value in the permutation poly should be one.
    // degree 2:
    // l_0(X) * (1 - z(X)) = 0
    //
    // The "last" value in the permutation poly should be a boolean, for
    // completeness and soundness.
    // degree 3:
    // l_last(X) * (z(X)^2 - z(X)) = 0
    //
    // Enable the permutation argument for only the rows involved.
    // degree (2 + input_degree + table_degree) or 4, whichever is larger:
    // (1 - (l_last(X) + l_blind(X))) * (
    //   z(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
    //   - z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
    // ) = 0
    //
    // The first two values of a' and s' should be the same.
    // degree 2:
    // l_0(X) * (a'(X) - s'(X)) = 0
    //
    // Either the two values are the same, or the previous
    // value of a' is the same as the current value.
    // degree 3:
    // (1 - (l_last(X) + l_blind(X))) * (a′(X) − s′(X))⋅(a′(X) − a′(\omega^{-1} X)) = 0
    let mut input_degree = 1;
    for expr in arg.input_expressions.iter() {
        input_degree = std::cmp::max(input_degree, expr.degree());
    }
    let mut table_degree = 1;
    for expr in arg.table_expressions.iter() {
        table_degree = std::cmp::max(table_degree, expr.degree());
    }

    // In practice because input_degree and table_degree are initialized to
    // one, the latter half of this max() invocation is at least 4 always,
    // rendering this call pointless except to be explicit in case we change
    // the initialization of input_degree/table_degree in the future.
    std::cmp::max(
        // (1 - (l_last + l_blind)) z(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
        4,
        // (1 - (l_last + l_blind)) z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
        2 + input_degree + table_degree,
    )
}

fn shuffle_argument_required_degree<F: Field, V: Variable>(arg: &shuffle::Argument<F, V>) -> usize {
    assert_eq!(arg.input_expressions.len(), arg.shuffle_expressions.len());

    let mut input_degree = 1;
    for expr in arg.input_expressions.iter() {
        input_degree = std::cmp::max(input_degree, expr.degree());
    }
    let mut shuffle_degree = 1;
    for expr in arg.shuffle_expressions.iter() {
        shuffle_degree = std::cmp::max(shuffle_degree, expr.degree());
    }

    // (1 - (l_last + l_blind)) (z(\omega X) (s(X) + \gamma) - z(X) (a(X) + \gamma))
    std::cmp::max(2 + shuffle_degree, 2 + input_degree)
}

use ff::PrimeField;
use halo2_middleware::circuit::{ColumnMid, VarMid};
use halo2_middleware::expression::{Expression, Variable};
use halo2_middleware::{lookup, shuffle};
use num_bigint::BigUint;
use std::collections::HashMap;
use std::fmt;

/// Wrapper type over `PrimeField` that implements Display with nice output.
/// - If the value is a power of two, format it as `2^k`
/// - If the value is smaller than 2^16, format it in decimal
/// - If the value is bigger than congruent -2^16, format it in decimal as the negative congruent
/// (between -2^16 and 0).
/// - Else format it in hex without leading zeros.
pub struct FDisp<'a, F: PrimeField>(pub &'a F);

impl<F: PrimeField> fmt::Display for FDisp<'_, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = (*self.0).to_repr();
        let v = v.as_ref();
        let v = BigUint::from_bytes_le(v);
        let v_bits = v.bits();
        if v_bits >= 8 && v.count_ones() == 1 {
            write!(f, "2^{}", v.trailing_zeros().unwrap_or_default())
        } else if v_bits < 16 {
            write!(f, "{}", v)
        } else {
            let v_neg = (F::ZERO - self.0).to_repr();
            let v_neg = v_neg.as_ref();
            let v_neg = BigUint::from_bytes_le(v_neg);
            let v_neg_bits = v_neg.bits();
            if v_neg_bits < 16 {
                write!(f, "-{}", v_neg)
            } else {
                write!(f, "0x{:x}", v)
            }
        }
    }
}

/// Wrapper type over `Expression` that implements Display with nice output.
/// The formatting of the `Expression::Variable` case is parametrized with the second field, which
/// take as auxiliary value the third field.
/// Use the constructor `expr_disp` to format variables using their `Display` implementation.
/// Use the constructor `expr_disp_names` for an `Expression` with `V: VarMid` to format column
/// queries according to their string names.
pub struct ExprDisp<'a, F: PrimeField, V: Variable, A>(
    /// Expression to display
    pub &'a Expression<F, V>,
    /// `V: Variable` formatter method that uses auxiliary value
    pub fn(&V, &mut fmt::Formatter<'_>, a: &A) -> fmt::Result,
    /// Auxiliary value to be passed to the `V: Variable` formatter
    pub &'a A,
);

fn var_fmt_default<V: Variable>(v: &V, f: &mut fmt::Formatter<'_>, _: &()) -> fmt::Result {
    write!(f, "{}", v)
}

fn var_fmt_names(
    v: &VarMid,
    f: &mut fmt::Formatter<'_>,
    names: &HashMap<ColumnMid, String>,
) -> fmt::Result {
    if let VarMid::Query(q) = v {
        if let Some(name) = names.get(&ColumnMid::new(q.column_type, q.column_index)) {
            return write!(f, "{}", name);
        }
    }
    write!(f, "{}", v)
}

/// ExprDisp constructor that formats viariables using their `Display` implementation.
pub fn expr_disp<F: PrimeField, V: Variable>(e: &Expression<F, V>) -> ExprDisp<F, V, ()> {
    ExprDisp(e, var_fmt_default, &())
}

/// ExprDisp constructor for an `Expression` with `V: VarMid` that formats column queries according
/// to their string names.
pub fn expr_disp_names<'a, F: PrimeField>(
    e: &'a Expression<F, VarMid>,
    names: &'a HashMap<ColumnMid, String>,
) -> ExprDisp<'a, F, VarMid, HashMap<ColumnMid, String>> {
    ExprDisp(e, var_fmt_names, names)
}

impl<F: PrimeField, V: Variable, A> fmt::Display for ExprDisp<'_, F, V, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_sum = |e: &Expression<F, V>| -> bool { matches!(e, Expression::Sum(_, _)) };
        let fmt_expr =
            |e: &Expression<F, V>, f: &mut fmt::Formatter<'_>, parens: bool| -> fmt::Result {
                if parens {
                    write!(f, "(")?;
                }
                write!(f, "{}", ExprDisp(e, self.1, self.2))?;
                if parens {
                    write!(f, ")")?;
                }
                Ok(())
            };

        match self.0 {
            Expression::Constant(c) => write!(f, "{}", FDisp(c)),
            Expression::Var(v) => self.1(v, f, self.2),
            Expression::Negated(a) => {
                write!(f, "-")?;
                fmt_expr(a, f, is_sum(a))
            }
            Expression::Sum(a, b) => {
                fmt_expr(a, f, false)?;
                if let Expression::Negated(neg) = &**b {
                    write!(f, " - ")?;
                    fmt_expr(neg, f, is_sum(neg))
                } else {
                    write!(f, " + ")?;
                    fmt_expr(b, f, false)
                }
            }
            Expression::Product(a, b) => {
                fmt_expr(a, f, is_sum(a))?;
                write!(f, " * ")?;
                fmt_expr(b, f, is_sum(b))
            }
        }
    }
}

/// Wrapper type over `lookup::Argument` that implements Display with nice output.
/// The formatting of the `Expression::Variable` case is parametrized with the second field, which
/// take as auxiliary value the third field.
/// Use the constructor `lookup_arg_disp` to format variables using their `Display` implementation.
/// Use the constructor `lookup_arg_disp_names` for a lookup of `Expression` with `V: VarMid` that
/// formats column queries according to their string names.
pub struct LookupArgDisp<'a, F: PrimeField, V: Variable, A>(
    /// Lookup argument to display
    pub &'a lookup::Argument<F, V>,
    /// `V: Variable` formatter method that uses auxiliary value
    pub fn(&V, &mut fmt::Formatter<'_>, a: &A) -> fmt::Result,
    /// Auxiliary value to be passed to the `V: Variable` formatter
    pub &'a A,
);

/// LookupArgDisp constructor that formats viariables using their `Display` implementation.
pub fn lookup_arg_disp<F: PrimeField, V: Variable>(
    a: &lookup::Argument<F, V>,
) -> LookupArgDisp<F, V, ()> {
    LookupArgDisp(a, var_fmt_default, &())
}

/// LookupArgDisp constructor for a lookup of `Expression` with `V: VarMid` that formats column
/// queries according to their string names.
pub fn lookup_arg_disp_names<'a, F: PrimeField>(
    a: &'a lookup::Argument<F, VarMid>,
    names: &'a HashMap<ColumnMid, String>,
) -> LookupArgDisp<'a, F, VarMid, HashMap<ColumnMid, String>> {
    LookupArgDisp(a, var_fmt_names, names)
}

impl<F: PrimeField, V: Variable, A> fmt::Display for LookupArgDisp<'_, F, V, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, expr) in self.0.input_expressions.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", ExprDisp(expr, self.1, self.2))?;
        }
        write!(f, "] in [")?;
        for (i, expr) in self.0.table_expressions.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", ExprDisp(expr, self.1, self.2))?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

/// Wrapper type over `shuffle::Argument` that implements Display with nice output.
/// The formatting of the `Expression::Variable` case is parametrized with the second field, which
/// take as auxiliary value the third field.
/// Use the constructor `shuffle_arg_disp` to format variables using their `Display`
/// implementation.
/// Use the constructor `shuffle_arg_disp_names` for a shuffle of `Expression` with `V: VarMid`
/// that formats column queries according to their string names.
pub struct ShuffleArgDisp<'a, F: PrimeField, V: Variable, A>(
    /// Shuffle argument to display
    pub &'a shuffle::Argument<F, V>,
    /// `V: Variable` formatter method that uses auxiliary value
    pub fn(&V, &mut fmt::Formatter<'_>, a: &A) -> fmt::Result,
    /// Auxiliary value to be passed to the `V: Variable` formatter
    pub &'a A,
);

/// ShuffleArgDisp constructor that formats viariables using their `Display` implementation.
pub fn shuffle_arg_disp<F: PrimeField, V: Variable>(
    a: &shuffle::Argument<F, V>,
) -> ShuffleArgDisp<F, V, ()> {
    ShuffleArgDisp(a, var_fmt_default, &())
}

/// ShuffleArgDisp constructor for a shuffle of `Expression` with `V: VarMid` that formats column
/// queries according to their string names.
pub fn shuffle_arg_disp_names<'a, F: PrimeField>(
    a: &'a shuffle::Argument<F, VarMid>,
    names: &'a HashMap<ColumnMid, String>,
) -> ShuffleArgDisp<'a, F, VarMid, HashMap<ColumnMid, String>> {
    ShuffleArgDisp(a, var_fmt_names, names)
}

impl<F: PrimeField, V: Variable, A> fmt::Display for ShuffleArgDisp<'_, F, V, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, expr) in self.0.input_expressions.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", ExprDisp(expr, self.1, self.2))?;
        }
        write!(f, "] shuff [")?;
        for (i, expr) in self.0.shuffle_expressions.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", ExprDisp(expr, self.1, self.2))?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ff::Field;
    use halo2_middleware::circuit::{Any, QueryMid, VarMid};
    use halo2_middleware::poly::Rotation;
    use halo2curves::bn256::Fr;

    #[test]
    fn test_lookup_shuffle_arg_disp() {
        type E = Expression<Fr, VarMid>;
        let a0 = VarMid::Query(QueryMid::new(Any::Advice, 0, Rotation(0)));
        let a1 = VarMid::Query(QueryMid::new(Any::Advice, 1, Rotation(0)));
        let f0 = VarMid::Query(QueryMid::new(Any::Fixed, 0, Rotation(0)));
        let a0: E = Expression::Var(a0);
        let a1: E = Expression::Var(a1);
        let f0: E = Expression::Var(f0);

        let names = [
            (ColumnMid::new(Any::Advice, 0), "a".to_string()),
            (ColumnMid::new(Any::Advice, 1), "b".to_string()),
            (ColumnMid::new(Any::Fixed, 0), "s".to_string()),
        ]
        .into_iter()
        .collect();

        let arg = lookup::Argument {
            name: "lookup".to_string(),
            input_expressions: vec![f0.clone() * a0.clone(), f0.clone() * a1.clone()],
            table_expressions: vec![f0.clone(), f0.clone() * (a0.clone() + a1.clone())],
        };
        assert_eq!(
            "[f0 * a0, f0 * a1] in [f0, f0 * (a0 + a1)]",
            format!("{}", lookup_arg_disp(&arg))
        );
        assert_eq!(
            "[s * a, s * b] in [s, s * (a + b)]",
            format!("{}", lookup_arg_disp_names(&arg, &names))
        );

        let arg = shuffle::Argument {
            name: "shuffle".to_string(),
            input_expressions: vec![f0.clone() * a0.clone(), f0.clone() * a1.clone()],
            shuffle_expressions: vec![f0.clone(), f0.clone() * (a0.clone() + a1.clone())],
        };
        assert_eq!(
            "[f0 * a0, f0 * a1] shuff [f0, f0 * (a0 + a1)]",
            format!("{}", shuffle_arg_disp(&arg))
        );
        assert_eq!(
            "[s * a, s * b] shuff [s, s * (a + b)]",
            format!("{}", shuffle_arg_disp_names(&arg, &names))
        );
    }

    #[test]
    fn test_expr_disp() {
        type E = Expression<Fr, VarMid>;
        let a0 = VarMid::Query(QueryMid::new(Any::Advice, 0, Rotation(0)));
        let a1 = VarMid::Query(QueryMid::new(Any::Advice, 1, Rotation(0)));
        let a0: E = Expression::Var(a0);
        let a1: E = Expression::Var(a1);

        let e = a0.clone() + a1.clone();
        assert_eq!("a0 + a1", format!("{}", expr_disp(&e)));
        let e = a0.clone() + a1.clone() + a0.clone();
        assert_eq!("a0 + a1 + a0", format!("{}", expr_disp(&e)));

        let e = a0.clone() * a1.clone();
        assert_eq!("a0 * a1", format!("{}", expr_disp(&e)));
        let e = a0.clone() * a1.clone() * a0.clone();
        assert_eq!("a0 * a1 * a0", format!("{}", expr_disp(&e)));

        let e = a0.clone() - a1.clone();
        assert_eq!("a0 - a1", format!("{}", expr_disp(&e)));
        let e = (a0.clone() - a1.clone()) - a0.clone();
        assert_eq!("a0 - a1 - a0", format!("{}", expr_disp(&e)));
        let e = a0.clone() - (a1.clone() - a0.clone());
        assert_eq!("a0 - (a1 - a0)", format!("{}", expr_disp(&e)));

        let e = a0.clone() * a1.clone() + a0.clone();
        assert_eq!("a0 * a1 + a0", format!("{}", expr_disp(&e)));
        let e = a0.clone() * (a1.clone() + a0.clone());
        assert_eq!("a0 * (a1 + a0)", format!("{}", expr_disp(&e)));

        let e = a0.clone() + a1.clone();
        let names = [
            (ColumnMid::new(Any::Advice, 0), "a".to_string()),
            (ColumnMid::new(Any::Advice, 1), "b".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!("a + b", format!("{}", expr_disp_names(&e, &names)));
    }

    #[test]
    fn test_f_disp() {
        let v = Fr::ZERO;
        assert_eq!("0", format!("{}", FDisp(&v)));

        let v = Fr::ONE;
        assert_eq!("1", format!("{}", FDisp(&v)));

        let v = Fr::from(12345u64);
        assert_eq!("12345", format!("{}", FDisp(&v)));

        let v = Fr::from(0x10000);
        assert_eq!("2^16", format!("{}", FDisp(&v)));

        let v = Fr::from(0x12345);
        assert_eq!("0x12345", format!("{}", FDisp(&v)));

        let v = -Fr::ONE;
        assert_eq!("-1", format!("{}", FDisp(&v)));

        let v = -Fr::from(12345u64);
        assert_eq!("-12345", format!("{}", FDisp(&v)));
    }
}

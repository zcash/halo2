use std::borrow::Borrow;
use std::ops::{Add, Mul, Neg, Sub};

use group::ff::Field;

use crate::plonk::{Assigned, Error};

/// A value that might exist within a circuit.
///
/// This behaves like `Option<V>` but differs in two key ways:
/// - It does not expose the enum cases, or provide an `Option::unwrap` equivalent. This
///   helps to ensure that unwitnessed values correctly propagate.
/// - It provides pass-through implementations of common traits such as `Add` and `Mul`,
///   for improved usability.
#[derive(Clone, Copy, Debug)]
pub struct Value<V> {
    inner: Option<V>,
}

impl<V> Default for Value<V> {
    fn default() -> Self {
        Self::unknown()
    }
}

impl<V> Value<V> {
    /// Constructs an unwitnessed value.
    #[must_use]
    pub const fn unknown() -> Self {
        Self { inner: None }
    }

    /// Constructs a known value.
    ///
    /// # Examples
    ///
    /// ```
    /// use halo2_frontend::circuit::Value;
    ///
    /// let v = Value::known(37);
    /// ```
    #[must_use]
    pub const fn known(value: V) -> Self {
        Self { inner: Some(value) }
    }

    /// Obtains the inner value for assigning into the circuit.
    ///
    /// Returns `Error::Synthesis` if this is [`Value::unknown()`].
    pub fn assign(self) -> Result<V, Error> {
        self.inner.ok_or(Error::Synthesis)
    }

    /// Converts from `&Value<V>` to `Value<&V>`.
    pub fn as_ref(&self) -> Value<&V> {
        Value {
            inner: self.inner.as_ref(),
        }
    }

    /// Converts from `&mut Value<V>` to `Value<&mut V>`.
    pub fn as_mut(&mut self) -> Value<&mut V> {
        Value {
            inner: self.inner.as_mut(),
        }
    }

    /// Enforces an assertion on the contained value, if known.
    ///
    /// The assertion is ignored if `self` is [`Value::unknown()`]. Do not try to enforce
    /// circuit constraints with this method!
    ///
    /// # Panics
    ///
    /// Panics if `f` returns `false`.
    pub fn assert_if_known<F: FnOnce(&V) -> bool>(&self, f: F) {
        if let Some(value) = self.inner.as_ref() {
            assert!(f(value));
        }
    }

    /// Checks the contained value for an error condition, if known.
    ///
    /// The error check is ignored if `self` is [`Value::unknown()`]. Do not try to
    /// enforce circuit constraints with this method!
    pub fn error_if_known_and<F: FnOnce(&V) -> bool>(&self, f: F) -> Result<(), Error> {
        match self.inner.as_ref() {
            Some(value) if f(value) => Err(Error::Synthesis),
            _ => Ok(()),
        }
    }

    /// Maps a `Value<V>` to `Value<W>` by applying a function to the contained value.
    pub fn map<W, F: FnOnce(V) -> W>(self, f: F) -> Value<W> {
        Value {
            inner: self.inner.map(f),
        }
    }

    /// Returns [`Value::unknown()`] if the value is [`Value::unknown()`], otherwise calls
    /// `f` with the wrapped value and returns the result.
    #[must_use]
    pub fn and_then<W, F: FnOnce(V) -> Value<W>>(self, f: F) -> Value<W> {
        match self.inner {
            Some(v) => f(v),
            None => Value::unknown(),
        }
    }

    /// Zips `self` with another `Value`.
    ///
    /// If `self` is `Value::known(s)` and `other` is `Value::known(o)`, this method
    /// returns `Value::known((s, o))`. Otherwise, [`Value::unknown()`] is returned.
    #[must_use]
    pub fn zip<W>(self, other: Value<W>) -> Value<(V, W)> {
        Value {
            inner: self.inner.zip(other.inner),
        }
    }
}

impl<V, W> Value<(V, W)> {
    /// Unzips a value containing a tuple of two values.
    ///
    /// If `self` is `Value::known((a, b)), this method returns
    /// `(Value::known(a), Value::known(b))`. Otherwise,
    /// `(Value::unknown(), Value::unknown())` is returned.
    #[must_use]
    pub fn unzip(self) -> (Value<V>, Value<W>) {
        match self.inner {
            Some((a, b)) => (Value::known(a), Value::known(b)),
            None => (Value::unknown(), Value::unknown()),
        }
    }
}

impl<V> Value<&V> {
    /// Maps a `Value<&V>` to a `Value<V>` by copying the contents of the value.
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn copied(self) -> Value<V>
    where
        V: Copy,
    {
        Value {
            inner: self.inner.copied(),
        }
    }

    /// Maps a `Value<&V>` to a `Value<V>` by cloning the contents of the value.
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn cloned(self) -> Value<V>
    where
        V: Clone,
    {
        Value {
            inner: self.inner.cloned(),
        }
    }
}

impl<V> Value<&mut V> {
    /// Maps a `Value<&mut V>` to a `Value<V>` by copying the contents of the value.
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn copied(self) -> Value<V>
    where
        V: Copy,
    {
        Value {
            inner: self.inner.copied(),
        }
    }

    /// Maps a `Value<&mut V>` to a `Value<V>` by cloning the contents of the value.
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn cloned(self) -> Value<V>
    where
        V: Clone,
    {
        Value {
            inner: self.inner.cloned(),
        }
    }
}

impl<V: Copy, const LEN: usize> Value<[V; LEN]> {
    /// Transposes a `Value<[V; LEN]>` into a `[Value<V>; LEN]`.
    ///
    /// [`Value::unknown()`] will be mapped to `[Value::unknown(); LEN]`.
    #[must_use]
    pub fn transpose_array(self) -> [Value<V>; LEN] {
        let mut ret = [Value::unknown(); LEN];
        if let Some(arr) = self.inner {
            for (entry, value) in ret.iter_mut().zip(arr) {
                *entry = Value::known(value);
            }
        }
        ret
    }
}

impl<V, I> Value<I>
where
    I: IntoIterator<Item = V>,
    I::IntoIter: ExactSizeIterator,
{
    /// Transposes a `Value<impl IntoIterator<Item = V>>` into a `Vec<Value<V>>`.
    ///
    /// [`Value::unknown()`] will be mapped to `vec![Value::unknown(); length]`.
    ///
    /// # Panics
    ///
    /// Panics if `self` is `Value::known(values)` and `values.len() != length`.
    #[must_use]
    pub fn transpose_vec(self, length: usize) -> Vec<Value<V>> {
        match self.inner {
            Some(values) => {
                let values = values.into_iter();
                assert_eq!(values.len(), length);
                values.map(Value::known).collect()
            }
            None => (0..length).map(|_| Value::unknown()).collect(),
        }
    }
}

//
// FromIterator
//

impl<A, V: FromIterator<A>> FromIterator<Value<A>> for Value<V> {
    /// Takes each element in the [`Iterator`]: if it is [`Value::unknown()`], no further
    /// elements are taken, and the [`Value::unknown()`] is returned. Should no
    /// [`Value::unknown()`] occur, a container of type `V` containing the values of each
    /// [`Value`] is returned.
    #[must_use]
    fn from_iter<I: IntoIterator<Item = Value<A>>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().map(|v| v.inner).collect(),
        }
    }
}

//
// Neg
//

impl<V: Neg> Neg for Value<V> {
    type Output = Value<V::Output>;

    #[must_use]
    fn neg(self) -> Self::Output {
        Value {
            inner: self.inner.map(|v| -v),
        }
    }
}

//
// Add
//

impl<V, O> Add for Value<V>
where
    V: Add<Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn add(self, rhs: Self) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a + b),
        }
    }
}

impl<V, O> Add for &Value<V>
where
    for<'v> &'v V: Add<Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn add(self, rhs: Self) -> Self::Output {
        Value {
            inner: self
                .inner
                .as_ref()
                .zip(rhs.inner.as_ref())
                .map(|(a, b)| a + b),
        }
    }
}

impl<V, O> Add<Value<&V>> for Value<V>
where
    for<'v> V: Add<&'v V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn add(self, rhs: Value<&V>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a + b),
        }
    }
}

impl<V, O> Add<Value<V>> for Value<&V>
where
    for<'v> &'v V: Add<V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn add(self, rhs: Value<V>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a + b),
        }
    }
}

impl<V, O> Add<&Value<V>> for Value<V>
where
    for<'v> V: Add<&'v V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn add(self, rhs: &Self) -> Self::Output {
        self + rhs.as_ref()
    }
}

impl<V, O> Add<Value<V>> for &Value<V>
where
    for<'v> &'v V: Add<V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn add(self, rhs: Value<V>) -> Self::Output {
        self.as_ref() + rhs
    }
}

//
// Sub
//

impl<V, O> Sub for Value<V>
where
    V: Sub<Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn sub(self, rhs: Self) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a - b),
        }
    }
}

impl<V, O> Sub for &Value<V>
where
    for<'v> &'v V: Sub<Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn sub(self, rhs: Self) -> Self::Output {
        Value {
            inner: self
                .inner
                .as_ref()
                .zip(rhs.inner.as_ref())
                .map(|(a, b)| a - b),
        }
    }
}

impl<V, O> Sub<Value<&V>> for Value<V>
where
    for<'v> V: Sub<&'v V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn sub(self, rhs: Value<&V>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a - b),
        }
    }
}

impl<V, O> Sub<Value<V>> for Value<&V>
where
    for<'v> &'v V: Sub<V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn sub(self, rhs: Value<V>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a - b),
        }
    }
}

impl<V, O> Sub<&Value<V>> for Value<V>
where
    for<'v> V: Sub<&'v V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn sub(self, rhs: &Self) -> Self::Output {
        self - rhs.as_ref()
    }
}

impl<V, O> Sub<Value<V>> for &Value<V>
where
    for<'v> &'v V: Sub<V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn sub(self, rhs: Value<V>) -> Self::Output {
        self.as_ref() - rhs
    }
}

//
// Mul
//

impl<V, O> Mul for Value<V>
where
    V: Mul<Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn mul(self, rhs: Self) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a * b),
        }
    }
}

impl<V, O> Mul for &Value<V>
where
    for<'v> &'v V: Mul<Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn mul(self, rhs: Self) -> Self::Output {
        Value {
            inner: self
                .inner
                .as_ref()
                .zip(rhs.inner.as_ref())
                .map(|(a, b)| a * b),
        }
    }
}

impl<V, O> Mul<Value<&V>> for Value<V>
where
    for<'v> V: Mul<&'v V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn mul(self, rhs: Value<&V>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a * b),
        }
    }
}

impl<V, O> Mul<Value<V>> for Value<&V>
where
    for<'v> &'v V: Mul<V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn mul(self, rhs: Value<V>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a * b),
        }
    }
}

impl<V, O> Mul<&Value<V>> for Value<V>
where
    for<'v> V: Mul<&'v V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn mul(self, rhs: &Self) -> Self::Output {
        self * rhs.as_ref()
    }
}

impl<V, O> Mul<Value<V>> for &Value<V>
where
    for<'v> &'v V: Mul<V, Output = O>,
{
    type Output = Value<O>;

    #[must_use]
    fn mul(self, rhs: Value<V>) -> Self::Output {
        self.as_ref() * rhs
    }
}

//
// Assigned<Field>
//

impl<F: Field> From<Value<F>> for Value<Assigned<F>> {
    #[must_use]
    fn from(value: Value<F>) -> Self {
        Self {
            inner: value.inner.map(Assigned::from),
        }
    }
}

impl<F: Field> Add<Value<F>> for Value<Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn add(self, rhs: Value<F>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a + b),
        }
    }
}

impl<F: Field> Add<F> for Value<Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn add(self, rhs: F) -> Self::Output {
        self + Value::known(rhs)
    }
}

impl<F: Field> Add<Value<F>> for Value<&Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn add(self, rhs: Value<F>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a + b),
        }
    }
}

impl<F: Field> Add<F> for Value<&Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn add(self, rhs: F) -> Self::Output {
        self + Value::known(rhs)
    }
}

impl<F: Field> Sub<Value<F>> for Value<Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn sub(self, rhs: Value<F>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a - b),
        }
    }
}

impl<F: Field> Sub<F> for Value<Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn sub(self, rhs: F) -> Self::Output {
        self - Value::known(rhs)
    }
}

impl<F: Field> Sub<Value<F>> for Value<&Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn sub(self, rhs: Value<F>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a - b),
        }
    }
}

impl<F: Field> Sub<F> for Value<&Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn sub(self, rhs: F) -> Self::Output {
        self - Value::known(rhs)
    }
}

impl<F: Field> Mul<Value<F>> for Value<Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn mul(self, rhs: Value<F>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a * b),
        }
    }
}

impl<F: Field> Mul<F> for Value<Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn mul(self, rhs: F) -> Self::Output {
        self * Value::known(rhs)
    }
}

impl<F: Field> Mul<Value<F>> for Value<&Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn mul(self, rhs: Value<F>) -> Self::Output {
        Value {
            inner: self.inner.zip(rhs.inner).map(|(a, b)| a * b),
        }
    }
}

impl<F: Field> Mul<F> for Value<&Assigned<F>> {
    type Output = Value<Assigned<F>>;

    #[must_use]
    fn mul(self, rhs: F) -> Self::Output {
        self * Value::known(rhs)
    }
}

impl<V> Value<V> {
    /// Returns the field element corresponding to this value.
    #[must_use]
    pub fn to_field<F: Field>(&self) -> Value<Assigned<F>>
    where
        for<'v> Assigned<F>: From<&'v V>,
    {
        Value {
            inner: self.inner.as_ref().map(|v| v.into()),
        }
    }

    /// Returns the field element corresponding to this value.
    #[must_use]
    pub fn into_field<F: Field>(self) -> Value<Assigned<F>>
    where
        V: Into<Assigned<F>>,
    {
        Value {
            inner: self.inner.map(|v| v.into()),
        }
    }

    /// Doubles this field element.
    ///
    /// # Examples
    ///
    /// If you have a `Value<F: Field>`, convert it to `Value<Assigned<F>>` first:
    /// ```
    /// # use halo2curves::pasta::pallas::Base as F;
    /// use halo2_frontend::circuit::Value;
    /// use halo2_frontend::plonk::Assigned;
    ///
    /// let v = Value::known(F::from(2));
    /// let v: Value<Assigned<F>> = v.into();
    /// let _ = v.double();
    /// ```
    #[must_use]
    pub fn double<F: Field>(&self) -> Value<Assigned<F>>
    where
        V: Borrow<Assigned<F>>,
    {
        Value {
            inner: self.inner.as_ref().map(|v| v.borrow().double()),
        }
    }

    /// Squares this field element.
    #[must_use]
    pub fn square<F: Field>(&self) -> Value<Assigned<F>>
    where
        V: Borrow<Assigned<F>>,
    {
        Value {
            inner: self.inner.as_ref().map(|v| v.borrow().square()),
        }
    }

    /// Cubes this field element.
    #[must_use]
    pub fn cube<F: Field>(&self) -> Value<Assigned<F>>
    where
        V: Borrow<Assigned<F>>,
    {
        Value {
            inner: self.inner.as_ref().map(|v| v.borrow().cube()),
        }
    }

    /// Inverts this assigned value (taking the inverse of zero to be zero).
    #[must_use]
    pub fn invert<F: Field>(&self) -> Value<Assigned<F>>
    where
        V: Borrow<Assigned<F>>,
    {
        Value {
            inner: self.inner.as_ref().map(|v| v.borrow().invert()),
        }
    }
}

impl<F: Field> Value<Assigned<F>> {
    /// Evaluates this value directly, performing an unbatched inversion if necessary.
    ///
    /// If the denominator is zero, the returned value is zero.
    #[must_use]
    pub fn evaluate(self) -> Value<F> {
        Value {
            inner: self.inner.map(|v| v.evaluate()),
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::op_ref)]

    use super::*;
    use halo2curves::bn256::Fr;

    type V = Value<i64>;

    impl PartialEq for V {
        fn eq(&self, other: &Self) -> bool {
            self.inner == other.inner
        }
    }
    impl PartialEq for Value<Assigned<Fr>> {
        fn eq(&self, other: &Self) -> bool {
            self.inner == other.inner
        }
    }

    #[test]
    fn test_value_as_mut() {
        let mut v_some = V::known(1);
        let mut v_none = V::default();
        v_some.as_mut().map(|v| *v = 3);
        v_none.as_mut().map(|v| *v = 3);
        assert_eq!(v_some, V::known(3));
        assert_eq!(v_none, V::unknown());
    }

    #[test]
    fn test_value_assert_if_known_ok() {
        V::known(1).assert_if_known(|v| *v == 1);
        V::unknown().assert_if_known(|v| *v == 1);
    }

    #[test]
    #[should_panic]
    fn test_value_assert_if_known_ko() {
        V::known(1).assert_if_known(|v| *v == 2);
    }

    #[test]
    fn test_value_error_if_known() {
        assert!(V::known(1).error_if_known_and(|v| *v == 1).is_err());
        assert!(V::known(1).error_if_known_and(|v| *v == 2).is_ok());
        assert!(V::unknown().error_if_known_and(|_| true).is_ok());
    }

    #[test]
    fn test_map() {
        assert_eq!(V::known(1).map(|v| v + 1), V::known(2));
        assert_eq!(V::unknown().map(|v| v + 1), V::unknown());
    }

    #[test]
    fn test_value_and_then() {
        let v = V::known(1);
        assert_eq!(v.and_then(|v| V::known(v + 1)), V::known(2));
        assert_eq!(v.and_then(|_| V::unknown()), V::unknown());
        assert_eq!(V::unknown().and_then(|v| V::known(v + 1)), V::unknown());
    }

    #[test]
    fn test_value_zip() {
        assert_eq!(
            V::known(1).zip(V::known(2)).unzip(),
            (V::known(1), V::known(2))
        );
        assert_eq!(
            V::known(1).zip(V::unknown()).unzip(),
            (V::unknown(), V::unknown())
        );
        assert_eq!(
            V::unknown().zip(V::known(2)).unzip(),
            (Value::unknown(), V::unknown())
        );
        assert_eq!(
            V::unknown().zip(V::unknown()).unzip(),
            (Value::unknown(), V::unknown())
        );
    }

    #[test]
    fn test_value_copies() {
        let copy = Value::<&mut i64>::known(&mut 1).copied();
        let clon = Value::<&mut i64>::known(&mut 1).cloned();
        assert_eq!(copy, Value::known(1));
        assert_eq!(clon, Value::known(1));
    }

    #[test]
    fn test_value_transpose_array() {
        assert_eq!(
            Value::<[_; 2]>::known([1, 2]).transpose_array(),
            [V::known(1), V::known(2)]
        );
    }

    #[test]
    fn test_value_transpose_vec_ok() {
        assert_eq!(
            Value::<[_; 2]>::known([1, 2]).transpose_vec(2),
            vec![V::known(1), V::known(2)]
        );
        assert_eq!(
            Value::<[_; 2]>::unknown().transpose_vec(2),
            vec![V::unknown(), V::unknown()]
        );

        // TODO: check if should be this allowed or not
        assert_eq!(
            Value::<[_; 6]>::unknown().transpose_vec(2),
            vec![V::unknown(), V::unknown()]
        );
    }

    #[test]
    #[should_panic]
    fn test_value_transpose_vec_ko_1() {
        assert_eq!(
            Value::<[_; 2]>::known([1, 2]).transpose_vec(1),
            vec![V::known(1), V::known(2)]
        );
    }

    #[test]
    #[should_panic]
    fn test_value_transpose_vec_ko_2() {
        assert_eq!(
            Value::<[_; 2]>::known([1, 2]).transpose_vec(3),
            vec![V::known(1), V::known(2)]
        );
    }

    #[test]
    fn test_value_from_iter() {
        assert_eq!(
            Value::<Vec<_>>::from_iter([V::known(1), V::known(2)]).inner,
            Some(vec![1, 2])
        );
        assert_eq!(
            Value::<Vec<_>>::from_iter([V::known(1), V::unknown()]).inner,
            None
        );
    }

    #[test]
    fn test_value_ops() {
        assert_eq!(-V::known(5), Value::known(-5));

        assert_eq!(V::known(5) + V::known(2), V::known(7));
        assert_eq!(&V::known(5) + V::known(2), V::known(7));
        assert_eq!(V::known(5) + &V::known(2), V::known(7));
        assert_eq!(&V::known(5) + &V::known(2), V::known(7));

        assert_eq!(V::known(5) - V::known(2), V::known(3));
        assert_eq!(&V::known(5) - V::known(2), V::known(3));
        assert_eq!(V::known(5) - &V::known(2), V::known(3));
        assert_eq!(&V::known(5) - &V::known(2), V::known(3));

        assert_eq!(V::known(5) * V::known(2), V::known(10));
        assert_eq!(&V::known(5) * V::known(2), V::known(10));
        assert_eq!(V::known(5) * &V::known(2), V::known(10));
        assert_eq!(&V::known(5) * &V::known(2), V::known(10));
    }

    #[test]
    fn test_value_assigned() {
        let fr_two = || Fr::from(2);
        let fr_three = || Fr::from(3);

        let one = Value::known(Assigned::Trivial(Fr::one()));
        let two = Value::known(Assigned::Trivial(Fr::from(2)));
        let six = Value::known(Assigned::Trivial(Fr::from(6)));

        let v: Value<Assigned<Fr>> = Value::known(Fr::one()).into();
        assert_eq!(v, Value::known(Assigned::Trivial(Fr::one())));

        assert_eq!(one + Fr::one(), two);
        assert_eq!(one + Value::known(Fr::one()), two);
        assert_eq!(
            Value::known(&Assigned::Trivial(Fr::one())) + Value::known(Fr::one()),
            two
        );
        assert_eq!(Value::known(&Assigned::Trivial(Fr::one())) + Fr::one(), two);

        assert_eq!(two - Value::known(Fr::one()), one);
        assert_eq!(two - Fr::one(), one);
        assert_eq!(
            Value::known(&Assigned::Trivial(fr_two())) - Value::known(Fr::one()),
            one
        );
        assert_eq!(Value::known(&Assigned::Trivial(fr_two())) - Fr::one(), one);

        assert_eq!(two * Value::known(fr_three()), six);
        assert_eq!(two * fr_three(), six);
        assert_eq!(
            Value::known(&Assigned::Trivial(fr_two())) * Value::known(fr_three()),
            six
        );
        assert_eq!(Value::known(&Assigned::Trivial(fr_two())) * fr_three(), six);
    }

    #[test]
    fn test_value_impl() {
        assert_eq!(
            Value::known(Fr::one()).to_field(),
            Value::known(Assigned::Trivial(Fr::one()))
        );
        assert_eq!(
            Value::known(Fr::one()).into_field(),
            Value::known(Assigned::Trivial(Fr::one()))
        );

        assert_eq!(
            Value::known(Assigned::Trivial(Fr::from(3))).double(),
            Value::known(Assigned::Trivial(Fr::from(6)))
        );
        assert_eq!(
            Value::known(Assigned::Trivial(Fr::from(3))).square(),
            Value::known(Assigned::Trivial(Fr::from(9)))
        );
        assert_eq!(
            Value::known(Assigned::Trivial(Fr::from(3))).cube(),
            Value::known(Assigned::Trivial(Fr::from(27)))
        );
        assert_eq!(
            Value::known(Assigned::Trivial(Fr::from(3)))
                .invert()
                .invert(),
            Value::known(Assigned::Trivial(Fr::from(3)))
        );
    }
}

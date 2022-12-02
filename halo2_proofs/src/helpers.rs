use crate::poly::Polynomial;
use ff::PrimeField;
use halo2curves::CurveAffine;
use std::io;

pub(crate) trait CurveRead: CurveAffine {
    /// Reads a compressed element from the buffer and attempts to parse it
    /// using `from_bytes`.
    fn read<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut compressed = Self::Repr::default();
        reader.read_exact(compressed.as_mut())?;
        Option::from(Self::from_bytes(&compressed))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid point encoding in proof"))
    }
}

impl<C: CurveAffine> CurveRead for C {}

pub(crate) trait SerdePrimeField: PrimeField {
    /// Reads a field element as bytes from the buffer using `from_repr`.
    /// Endianness is specified by `PrimeField` implementation.
    fn read<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut compressed = Self::Repr::default();
        reader.read_exact(compressed.as_mut())?;
        Option::from(Self::from_repr(compressed)).ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Invalid prime field point encoding")
        })
    }

    /// Writes a field element as bytes to the buffer using `to_repr`.
    /// Endianness is specified by `PrimeField` implementation.
    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self.to_repr().as_ref())
    }
}

impl<F: PrimeField> SerdePrimeField for F {}

/// Convert a slice of `bool` into a `u8`.
///
/// Panics if the slice has length greater than 8.
pub fn pack(bits: &[bool]) -> u8 {
    let mut value = 0u8;
    assert!(bits.len() <= 8);
    for (bit_index, bit) in bits.iter().enumerate() {
        value |= (*bit as u8) << bit_index;
    }
    value
}

/// Writes the first `bits.len()` bits of a `u8` into `bits`.
pub fn unpack(byte: u8, bits: &mut [bool]) {
    for (bit_index, bit) in bits.iter_mut().enumerate() {
        *bit = (byte >> bit_index) & 1 == 1;
    }
}

/// Reads a vector of polynomials from buffer
pub(crate) fn read_polynomial_vec<R: io::Read, F: PrimeField, B>(
    reader: &mut R,
) -> io::Result<Vec<Polynomial<F, B>>> {
    let mut len_be_bytes = [0u8; 4];
    reader.read_exact(&mut len_be_bytes)?;
    let len = u32::from_be_bytes(len_be_bytes);

    (0..len)
        .map(|_| Polynomial::<F, B>::read(reader))
        .collect::<io::Result<Vec<_>>>()
}

/// Writes a slice of polynomials to buffer
pub(crate) fn write_polynomial_slice<W: io::Write, F: PrimeField, B>(
    slice: &[Polynomial<F, B>],
    writer: &mut W,
) -> io::Result<()> {
    writer.write_all(&(slice.len() as u32).to_be_bytes())?;
    for poly in slice.iter() {
        poly.write(writer)?;
    }
    Ok(())
}

/// Gets the total number of bytes of a slice of polynomials, assuming all polynomials are the same length
pub(crate) fn polynomial_slice_byte_length<F: PrimeField, B>(slice: &[Polynomial<F, B>]) -> usize {
    let field_len = F::default().to_repr().as_ref().len();
    4 + slice.len() * (4 + field_len * slice.get(0).map(|poly| poly.len()).unwrap_or(0))
}

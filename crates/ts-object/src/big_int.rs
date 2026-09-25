
use std::{ cmp::Ordering,
           error::Error,
           fmt::{ self, Display, Formatter },
           ops::{ Add, Mul, Neg, Sub },
           str::FromStr };



/**
 * Arbitrary precision signed integer.
 *
 * Magnitude is stored as little-endian base-2^64 limbs.
 * Zero is always canonicalized to:
 *
 *     negative = false
 *     limbs = []
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BigInt
{
    negative: bool,
    limbs: Vec<u64>
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseBigIntError;


impl Display for ParseBigIntError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "Invalid BigInt.")
    }
}


impl Error for ParseBigIntError
{
}


impl BigInt
{
    pub fn zero() -> Self
    {
        Self
        {
            negative: false,
            limbs: Vec::new()
        }
    }


    pub fn is_zero(&self) -> bool
    {
        self.limbs.is_empty()
    }


    pub fn is_negative(&self) -> bool
    {
        self.negative
    }


    fn from_parts(negative: bool, limbs: Vec<u64>) -> Self
    {
        let mut value = Self
        {
            negative,
            limbs
        };

        value.normalize();

        value
    }


    fn normalize(&mut self)
    {
        while self.limbs.last() == Some(&0)
        {
            self.limbs.pop();
        }

        if self.limbs.is_empty()
        {
            self.negative = false;
        }
    }


    fn compare_absolute(&self, other: &Self) -> Ordering
    {
        match self.limbs.len().cmp(&other.limbs.len())
        {
            Ordering::Equal => {},
            ordering => return ordering
        }

        for (left, right) in self.limbs.iter().rev().zip(other.limbs.iter().rev())
        {
            match left.cmp(right)
            {
                Ordering::Equal => {},
                ordering => return ordering
            }
        }

        Ordering::Equal
    }


    fn add_absolute(left: &Self, right: &Self) -> Vec<u64>
    {
        let length = left.limbs.len().max(right.limbs.len());

        let mut result = Vec::with_capacity(length + 1);
        let mut carry = 0u128;

        for index in 0..length
        {
            let left_limb = left.limbs.get(index).copied().unwrap_or(0) as u128;
            let right_limb = right.limbs.get(index).copied().unwrap_or(0) as u128;

            let value = left_limb + right_limb + carry;

            result.push(value as u64);
            carry = value >> 64;
        }

        if carry != 0
        {
            result.push(carry as u64);
        }

        result
    }


    /**
     * Subtract magnitudes.
     *
     * `left` must be greater than or equal to `right`.
     */
    fn subtract_absolute(left: &Self, right: &Self) -> Vec<u64>
    {
        debug_assert!(left.compare_absolute(right) != Ordering::Less);

        let mut result = Vec::with_capacity(left.limbs.len());
        let mut borrow = false;

        for index in 0..left.limbs.len()
        {
            let left_limb = left.limbs[index];
            let right_limb = right.limbs.get(index).copied().unwrap_or(0);

            let (value, borrow_a) = left_limb.overflowing_sub(right_limb);
            let (value, borrow_b) = value.overflowing_sub(borrow as u64);

            result.push(value);

            borrow = borrow_a || borrow_b;
        }

        debug_assert!(!borrow);

        result
    }


    fn add_values(left: &Self, right: &Self) -> Self
    {
        if left.negative == right.negative
        {
            return Self::from_parts(left.negative,Self::add_absolute(left, right));
        }

        match left.compare_absolute(right)
        {
            Ordering::Greater => Self::from_parts(left.negative,Self::subtract_absolute(left,
                                                                                        right)),

            Ordering::Less => Self::from_parts(right.negative,Self::subtract_absolute(right, left)),

            Ordering::Equal => Self::zero()
        }
    }


    fn multiply_values(left: &Self, right: &Self) -> Self
    {
        if left.is_zero() || right.is_zero()
        {
            return Self::zero();
        }

        let mut result =
            vec![0u64; left.limbs.len() + right.limbs.len()];

        for (left_index, &left_limb) in left.limbs.iter().enumerate()
        {
            let mut carry = 0u128;

            for (right_index, &right_limb) in right.limbs.iter().enumerate()
            {
                let index = left_index + right_index;

                let value =
                    result[index] as u128
                    + (left_limb as u128 * right_limb as u128)
                    + carry;

                result[index] = value as u64;
                carry = value >> 64;
            }

            let mut index = left_index + right.limbs.len();

            while carry != 0
            {
                if index == result.len()
                {
                    result.push(0);
                }

                let value = result[index] as u128 + carry;

                result[index] = value as u64;
                carry = value >> 64;

                index += 1;
            }
        }

        Self::from_parts(
            left.negative != right.negative,
            result
        )
    }


    fn multiply_small(&mut self, multiplier: u64)
    {
        if multiplier == 0 || self.is_zero()
        {
            *self = Self::zero();
            return;
        }

        let mut carry = 0u128;

        for limb in &mut self.limbs
        {
            let value =
                (*limb as u128 * multiplier as u128)
                + carry;

            *limb = value as u64;
            carry = value >> 64;
        }

        if carry != 0
        {
            self.limbs.push(carry as u64);
        }
    }


    fn add_small(&mut self, value: u64)
    {
        if value == 0
        {
            return;
        }

        if self.is_zero()
        {
            self.limbs.push(value);
            return;
        }

        let mut carry = value as u128;
        let mut index = 0;

        while carry != 0
        {
            if index == self.limbs.len()
            {
                self.limbs.push(0);
            }

            let value =
                self.limbs[index] as u128
                + carry;

            self.limbs[index] = value as u64;
            carry = value >> 64;

            index += 1;
        }
    }


    /**
     * Divide the magnitude by a machine-sized integer.
     *
     * Returns the remainder.
     */
    fn divide_small(&mut self, divisor: u64) -> u64
    {
        debug_assert!(divisor != 0);

        let mut remainder = 0u128;

        for limb in self.limbs.iter_mut().rev()
        {
            let value =
                (remainder << 64)
                | *limb as u128;

            *limb = (value / divisor as u128) as u64;
            remainder = value % divisor as u128;
        }

        self.normalize();

        remainder as u64
    }
}


impl FromStr for BigInt
{
    type Err = ParseBigIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err>
    {
        if value.is_empty()
        {
            return Err(ParseBigIntError);
        }

        let bytes = value.as_bytes();

        let mut index = 0;
        let mut negative = false;

        match bytes[0]
        {
            b'-' =>
                {
                    negative = true;
                    index = 1;
                },

            b'+' =>
                {
                    index = 1;
                },

            _ => {}
        }

        if index == bytes.len()
        {
            return Err(ParseBigIntError);
        }

        let mut result = BigInt::zero();

        while index < bytes.len()
        {
            let byte = bytes[index];

            if !byte.is_ascii_digit()
            {
                return Err(ParseBigIntError);
            }

            result.multiply_small(10);
            result.add_small((byte - b'0') as u64);

            index += 1;
        }

        if !result.is_zero()
        {
            result.negative = negative;
        }

        Ok(result)
    }
}


impl Display for BigInt
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        if self.is_zero()
        {
            return write!(f, "0");
        }

        if self.negative
        {
            write!(f, "-")?;
        }

        const DECIMAL_BASE: u64 =
            10_000_000_000_000_000_000;

        let mut value = self.clone();
        value.negative = false;

        let mut chunks = Vec::new();

        while !value.is_zero()
        {
            chunks.push(value.divide_small(DECIMAL_BASE));
        }

        let first = chunks.pop().unwrap();

        write!(f, "{}", first)?;

        while let Some(chunk) = chunks.pop()
        {
            write!(f, "{:019}", chunk)?;
        }

        Ok(())
    }
}


impl Ord for BigInt
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        match (self.negative, other.negative)
        {
            (true, false) =>
                Ordering::Less,

            (false, true) =>
                Ordering::Greater,

            (false, false) =>
                self.compare_absolute(other),

            (true, true) =>
                self.compare_absolute(other).reverse()
        }
    }
}


impl PartialOrd for BigInt
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}


impl Neg for BigInt
{
    type Output = BigInt;

    fn neg(mut self) -> Self::Output
    {
        if !self.is_zero()
        {
            self.negative = !self.negative;
        }

        self
    }
}


impl Add<&BigInt> for &BigInt
{
    type Output = BigInt;

    fn add(self, other: &BigInt) -> Self::Output
    {
        BigInt::add_values(self, other)
    }
}


impl Sub<&BigInt> for &BigInt
{
    type Output = BigInt;

    fn sub(self, other: &BigInt) -> Self::Output
    {
        let mut other = other.clone();

        if !other.is_zero()
        {
            other.negative = !other.negative;
        }

        BigInt::add_values(self, &other)
    }
}


impl Mul<&BigInt> for &BigInt
{
    type Output = BigInt;

    fn mul(self, other: &BigInt) -> Self::Output
    {
        BigInt::multiply_values(self, other)
    }
}


impl From<u64> for BigInt
{
    fn from(value: u64) -> Self
    {
        if value == 0
        {
            Self::zero()
        }
        else
        {
            Self::from_parts(false, vec![value])
        }
    }
}


impl From<i64> for BigInt
{
    fn from(value: i64) -> Self
    {
        if value == 0
        {
            return Self::zero();
        }

        Self::from_parts(
            value.is_negative(),
            vec![value.unsigned_abs()]
        )
    }
}


impl From<u128> for BigInt
{
    fn from(value: u128) -> Self
    {
        if value == 0
        {
            return Self::zero();
        }

        let low = value as u64;
        let high = (value >> 64) as u64;

        let mut limbs = vec![low];

        if high != 0
        {
            limbs.push(high);
        }

        Self::from_parts(false, limbs)
    }
}


impl From<i128> for BigInt
{
    fn from(value: i128) -> Self
    {
        if value == 0
        {
            return Self::zero();
        }

        let magnitude = value.unsigned_abs();

        let mut result = Self::from(magnitude);
        result.negative = value.is_negative();

        result
    }
}

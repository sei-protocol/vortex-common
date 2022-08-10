use crate::error::ContractError;
use cosmwasm_std::{Decimal, DecimalRangeExceeded, Fraction, Uint128};
use cosmwasm_std::{Deps, StdError};
use forward_ref::{forward_ref_binop, forward_ref_op_assign};
use schemars::JsonSchema;
use sei_cosmwasm::SeiQueryWrapper;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use std::{fmt, ops::BitXor};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, JsonSchema, Debug, Eq)]
pub struct SignedDecimal {
    pub decimal: Decimal,
    pub negative: bool,
}

impl SignedDecimal {
    pub const fn zero() -> Self {
        SignedDecimal {
            decimal: Decimal::zero(),
            negative: false,
        }
    }
    pub const fn one() -> Self {
        SignedDecimal {
            decimal: Decimal::one(),
            negative: false,
        }
    }

    pub const fn new(decimal: Decimal) -> Self {
        SignedDecimal {
            decimal: decimal,
            negative: false,
        }
    }

    pub const fn new_from_ptr(decimal: &Decimal) -> Self {
        SignedDecimal {
            decimal: *decimal,
            negative: false,
        }
    }

    pub const fn new_negative(decimal: Decimal) -> Self {
        SignedDecimal {
            decimal: decimal,
            negative: true,
        }
    }

    pub const fn new_signed(decimal: Decimal, negative: bool) -> Self {
        SignedDecimal {
            decimal: decimal,
            negative: negative,
        }
    }

    pub fn from_atomics(
        atomics: impl Into<Uint128>,
        decimal_places: u32,
        negative: bool,
    ) -> Result<Self, DecimalRangeExceeded> {
        match Decimal::from_atomics(atomics, decimal_places) {
            Ok(decimal) => Result::Ok(SignedDecimal {
                decimal: decimal,
                negative: negative,
            }),
            Err(err) => Result::Err(err),
        }
    }

    pub fn negation(&self) -> Self {
        if self.decimal == Decimal::zero() {
            return *self;
        }
        return SignedDecimal {
            decimal: self.decimal,
            negative: !self.negative,
        };
    }

    pub fn is_zero(&self) -> bool {
        self.decimal == Decimal::zero()
    }

    pub fn positive_part(&self) -> SignedDecimal {
        if self.negative {
            return SignedDecimal::zero();
        }
        *self
    }
}

impl Ord for SignedDecimal {
    fn cmp(&self, other: &SignedDecimal) -> Ordering {
        if self.negative && other.negative {
            if self.decimal > other.decimal {
                Ordering::Less
            } else if self.decimal == other.decimal {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        } else if !self.negative && !other.negative {
            if self.decimal < other.decimal {
                Ordering::Less
            } else if self.decimal == other.decimal {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        } else if !self.negative && other.negative {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for SignedDecimal {
    fn partial_cmp(&self, other: &SignedDecimal) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for SignedDecimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.negative && other.negative {
            SignedDecimal {
                decimal: self.decimal + other.decimal,
                negative: true,
            }
        } else if self.negative && !other.negative {
            if self.decimal > other.decimal {
                SignedDecimal {
                    decimal: self.decimal - other.decimal,
                    negative: true,
                }
            } else {
                SignedDecimal {
                    decimal: other.decimal - self.decimal,
                    negative: false,
                }
            }
        } else if !self.negative && other.negative {
            if self.decimal >= other.decimal {
                SignedDecimal {
                    decimal: self.decimal - other.decimal,
                    negative: false,
                }
            } else {
                SignedDecimal {
                    decimal: other.decimal - self.decimal,
                    negative: true,
                }
            }
        } else {
            assert_eq!(!self.negative && !other.negative, true);
            SignedDecimal {
                decimal: self.decimal + other.decimal,
                negative: false,
            }
        }
    }
}
forward_ref_binop!(impl Add, add for SignedDecimal, SignedDecimal);

impl AddAssign for SignedDecimal {
    fn add_assign(&mut self, rhs: SignedDecimal) {
        *self = *self + rhs;
    }
}
forward_ref_op_assign!(impl AddAssign, add_assign for SignedDecimal, SignedDecimal);

impl Sub for SignedDecimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if other.decimal == Decimal::zero() {
            return self;
        }
        self + SignedDecimal {
            decimal: other.decimal,
            negative: !other.negative,
        }
    }
}
forward_ref_binop!(impl Sub, sub for SignedDecimal, SignedDecimal);

impl SubAssign for SignedDecimal {
    fn sub_assign(&mut self, rhs: SignedDecimal) {
        *self = *self - rhs;
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for SignedDecimal, SignedDecimal);

impl Mul for SignedDecimal {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Self) -> Self {
        if (self.negative && other.negative) || (!self.negative && !other.negative) {
            SignedDecimal {
                decimal: self.decimal * other.decimal,
                negative: false,
            }
        } else {
            let mut is_result_negative = true;
            if self.decimal == Decimal::zero() || other.decimal == Decimal::zero() {
                is_result_negative = false
            }
            SignedDecimal {
                decimal: self.decimal * other.decimal,
                negative: is_result_negative,
            }
        }
    }
}

impl Fraction<Uint128> for SignedDecimal {
    #[inline]
    fn numerator(&self) -> Uint128 {
        self.decimal.numerator()
    }

    #[inline]
    fn denominator(&self) -> Uint128 {
        self.decimal.denominator()
    }

    /// Returns the multiplicative inverse `1/d` for decimal `d`.
    ///
    /// If `d` is zero, none is returned.
    fn inv(&self) -> Option<Self> {
        self.decimal.inv().map(|d| SignedDecimal {
            decimal: d,
            negative: self.negative,
        })
    }
}

/// SignedDecimal / SignedDecimal
impl Div for SignedDecimal {
    // The division of signeddecimal is a closed operation.
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.decimal.is_zero() {
            panic!("Cannot divide by zero-valued `SignedDecimal`!");
        }
        let reciprocal = rhs.decimal.inv().unwrap();
        let decimal_res = match reciprocal.checked_mul(self.decimal) {
            Ok(res) => res,
            Err(e) => panic!("{}", e),
        };
        match self.negative.bitxor(rhs.negative) {
            true => Self::new_negative(decimal_res),
            false => Self::new(decimal_res),
        }
    }
}

impl fmt::Display for SignedDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negative {
            write!(f, "-{}", self.decimal)
        } else {
            write!(f, "{}", self.decimal)
        }
    }
}

fn epsilon() -> Decimal {
    Decimal::from_atomics(1u128, 8).unwrap()
}

pub fn roughly_equal(d1: Decimal, d2: Decimal) -> bool {
    roughly_equal_signed(SignedDecimal::new(d1), SignedDecimal::new(d2))
}

pub fn roughly_equal_signed(d1: SignedDecimal, d2: SignedDecimal) -> bool {
    (d1 - d2).decimal < epsilon()
}

// convert decimal to uint128, conservative round down
pub fn decimal2uint128_floor(d: Decimal) -> Uint128 {
    let base: u64 = 10; // to avoid overflow with 10^18
    let atomics = d.atomics();
    let decimal_places = d.decimal_places();
    atomics / Uint128::new(base.pow(decimal_places) as u128)
}

pub fn decimal2u128_floor(d: Decimal) -> u128 {
    let base: u64 = 10; // to avoid overflow with 10^18
    let atomics = d.atomics();
    let decimal_places = d.decimal_places();
    atomics.u128() / base.pow(decimal_places) as u128
}

pub fn decimal2u128_ceiling(d: Decimal) -> u128 {
    let base: u64 = 10; // to avoid overflow with 10^18
    let atomics = d.atomics();
    let decimal_places = d.decimal_places();
    let divisor = base.pow(decimal_places) as u128;
    (atomics.u128() + divisor - 1) / divisor
}

pub fn validate_migration(
    deps: Deps<SeiQueryWrapper>,
    contract_name: &str,
    contract_version: &str,
) -> Result<(), ContractError> {
    let ver = cw2::get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != contract_name {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }

    let storage_version: Version = ver.version.parse()?;
    let version: Version = contract_version.parse()?;
    if storage_version >= version {
        return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    }
    Ok(())
}

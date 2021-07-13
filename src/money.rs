use std::fmt;
use num::Zero;
use num::rational::{Rational64, ParseRatioError};
use serde::de::{self, Visitor, Unexpected};

use std::cmp::{Ordering, PartialOrd};
use std::ops::{Add, AddAssign, Deref, SubAssign};
use std::str::FromStr;
use std::convert::TryInto;

use serde::{Serialize, Serializer};

impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where 
        S: Serializer
    {
        let numer = *self.numer() as f64;
        let denom = *self.denom() as f64;
        let val = (numer / denom) * 10_000.0 / 10_000.0;
        serializer.serialize_str(format!("{}", val).as_str())
    }
}

// naive https://ren.zone/articles/safe-money sketch
// pub type Money = Rational64;
#[derive(Copy, Clone, Debug)]
pub struct Money(Rational64);

impl Money {
    pub fn zero() -> Self {
        Money(Rational64::zero())        
    }
    pub fn from_str(s: &str) -> Result<Money, ParseRatioError> {
        Ok(Money(Rational64::from_str(s)?))
    }
}

// Enable `Deref` coercion.
impl Deref for Money {
    type Target = Rational64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add for Money {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Money(*self + *other)
    }
}
impl AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        *self = Money(**self + *other);
    }
}
impl SubAssign for Money {
    fn sub_assign(&mut self, other: Self) {
        *self = Money(**self - *other);
    }
}
impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Money {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

pub struct MoneyVisitor;

impl<'de> Visitor<'de> for MoneyVisitor {
    type Value = Option<Money>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representation of f64")
    }
    fn visit_str<E>(self, value: &str) -> Result<Option<Money>, E> 
    where 
        E: de::Error 
    {
        let to_invalid_value_err = |_| E::invalid_value(
            Unexpected::Str(value), &"a string representation of a f64"
        );

        match value.find('.') {
            Some(i) => {
                let (numer, denum) = (&value[0..i], &value[i+1..]);
                let integral = format!("{}/1", numer);
                let integral = Money::from_str(integral.as_str()).map_err(to_invalid_value_err)?;
                
                let frac_size: usize = match denum.len() {
                    1..=4 => denum.len(),
                    _ => 4
                };
                let frac_denum = i32::pow(10, frac_size.try_into().unwrap());
                let fraction = format!("{}/{}", &denum[..=frac_size-1], frac_denum);
                let fraction = Money::from_str(fraction.as_str()).map_err(to_invalid_value_err)?;

                Ok(Some(integral + fraction))
            }
            None => {
                match value.len() {
                    0 => Ok(None),
                    _ => {
                        let integral = format!("{}/1", value);
                        Money::from_str(integral.as_str()).map_err(to_invalid_value_err).map(Some)
                    }
                }
            }
        }
    }
}
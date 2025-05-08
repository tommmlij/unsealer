use std::ops::Deref;
use std::str::FromStr;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

use sodiumoxide::crypto::box_::{PUBLICKEYBYTES, PublicKey, SECRETKEYBYTES, SecretKey};

pub trait FromBase64Sized<const N: usize>: Sized {
    fn from_bytes(bytes: [u8; N]) -> Self;
}

#[derive(Debug, Clone)]
pub struct Base64Key<T, const N: usize>(pub T);

impl<T, const N: usize> FromStr for Base64Key<T, N>
where
    T: FromBase64Sized<N>,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = URL_SAFE.decode(s).map_err(|e| format!("Invalid base64: {}", e))?;
        if decoded.len() != N {
            return Err(format!("Expected {} bytes, got {}", N, decoded.len()));
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(&decoded);
        Ok(Base64Key(T::from_bytes(arr)))
    }
}

impl<T, const N: usize> Deref for Base64Key<T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromBase64Sized<SECRETKEYBYTES> for SecretKey {
    fn from_bytes(bytes: [u8; SECRETKEYBYTES]) -> Self {
        SecretKey(bytes)
    }
}

impl FromBase64Sized<PUBLICKEYBYTES> for PublicKey {
    fn from_bytes(bytes: [u8; PUBLICKEYBYTES]) -> Self {
        PublicKey(bytes)
    }
}
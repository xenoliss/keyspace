use crate::Hash256;
use anyhow::Result;

/// Trait that provides a method to verify the validity of an imt mutation.
pub trait ImtMutate<H> {
    /// Verifies the imt mutation.
    /// The returned result contains the new imt root if the verification passed, else an error is returned.
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256>;
}

impl<H, T> ImtMutate<H> for &T
where
    T: ImtMutate<H>,
{
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        T::verify(*self, hasher_factory, old_root)
    }
}

impl<H, T> ImtMutate<H> for &mut T
where
    T: ImtMutate<H>,
{
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        T::verify(*self, hasher_factory, old_root)
    }
}

impl<H, T> ImtMutate<H> for Box<T>
where
    T: ImtMutate<H>,
{
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        T::verify(self, hasher_factory, old_root)
    }
}

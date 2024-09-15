use crate::Hash256;
use anyhow::Result;

/// Trait that provides a method to verify the validity of an IMT mutation.
pub trait IMTMutate<H> {
    /// Verifies the IMT mutation.
    ///
    /// The returned result contains the new IMT root if the verification passed, else an error is returned.
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256>;
}

impl<H, T> IMTMutate<H> for &T
where
    T: IMTMutate<H>,
{
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        T::verify(*self, hasher_factory, old_root)
    }
}

impl<H, T> IMTMutate<H> for &mut T
where
    T: IMTMutate<H>,
{
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        T::verify(*self, hasher_factory, old_root)
    }
}

impl<H, T> IMTMutate<H> for Box<T>
where
    T: IMTMutate<H>,
{
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        T::verify(self, hasher_factory, old_root)
    }
}

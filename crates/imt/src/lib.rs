#![feature(trait_alias)]

use tiny_keccak::Hasher;

pub mod node;
pub mod proof;
pub mod storage;
pub mod tree;

pub type Hash256 = [u8; 32];

pub trait NodeKey = Default + Clone + Copy + Ord + AsRef<[u8]>;
pub trait NodeValue = Default + Clone + Copy + AsRef<[u8]>;

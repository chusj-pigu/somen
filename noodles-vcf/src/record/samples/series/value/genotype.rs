mod iter;

use std::io;

use self::iter::Iter;
use crate::variant::record::samples::series::value::genotype::Phasing;

/// VCF record samples series genotype value.
#[derive(Debug)]
pub struct Genotype<'a>(&'a str);

impl<'a> Genotype<'a> {
    pub(crate) fn new(src: &'a str) -> Self {
        Self(src)
    }
}

impl crate::variant::record::samples::series::value::Genotype for Genotype<'_> {
    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<(Option<usize>, Phasing)>> + '_> {
        Box::new(Iter::new(self.0))
    }
}

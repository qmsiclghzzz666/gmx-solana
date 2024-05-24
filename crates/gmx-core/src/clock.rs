/// The kind of clocks.
#[derive(Debug, Clone, Copy, num_enum::TryFromPrimitive, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum ClockKind {
    /// Price Impact Distribution.
    PriceImpactDistribution,
}

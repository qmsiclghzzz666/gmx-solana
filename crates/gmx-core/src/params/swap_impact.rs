/// Swap impact parameters.
#[derive(Debug, Clone, Copy)]
pub struct SwapImpactParams<T> {
    exponent: T,
    positive_factor: T,
    negative_factor: T,
}

impl<T> SwapImpactParams<T> {
    /// Exponent.
    pub fn exponent(&self) -> &T {
        &self.exponent
    }

    /// Positive factor.
    pub fn positive_factor(&self) -> &T {
        &self.positive_factor
    }

    /// Negative factor.
    pub fn negative_factor(&self) -> &T {
        &self.negative_factor
    }

    /// Builder.
    pub fn builder() -> Builder<T> {
        Builder {
            exponent: None,
            positive_factor: None,
            negative_factor: None,
        }
    }
}

/// Builder for Swap impact parameters.
pub struct Builder<T> {
    exponent: Option<T>,
    positive_factor: Option<T>,
    negative_factor: Option<T>,
}

impl<T> Builder<T> {
    /// Set `exponent`
    pub fn with_exponent(mut self, exponent: T) -> Self {
        self.exponent = Some(exponent);
        self
    }

    /// Set `positive_factor`
    pub fn with_positive_factor(mut self, factor: T) -> Self {
        self.positive_factor = Some(factor);
        self
    }

    /// Set `negative_factor`
    pub fn with_negative_factor(mut self, factor: T) -> Self {
        self.negative_factor = Some(factor);
        self
    }

    /// Build [`SwapImpactParams`].
    pub fn build(self) -> crate::Result<SwapImpactParams<T>> {
        Ok(SwapImpactParams {
            exponent: self
                .exponent
                .ok_or(crate::Error::build_params("missing `exponent`"))?,
            positive_factor: self
                .positive_factor
                .ok_or(crate::Error::build_params("missing `positive_factor"))?,
            negative_factor: self
                .negative_factor
                .ok_or(crate::Error::build_params("missing `negative_factor`"))?,
        })
    }
}

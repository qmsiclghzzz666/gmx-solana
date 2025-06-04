#[cfg(feature = "gmsol-utils")]
mod utils {
    use gmsol_utils::{
        fixed_str::bytes_to_fixed_str,
        impl_flags,
        instruction::{InstructionFlag, MAX_IX_FLAGS},
    };

    use crate::gmsol_timelock::{accounts::Executor, types::InstructionFlagContainer};

    impl Executor {
        /// Get role name.
        pub fn role_name(&self) -> crate::Result<&str> {
            bytes_to_fixed_str(&self.role_name).map_err(crate::Error::custom)
        }
    }

    impl_flags!(InstructionFlag, MAX_IX_FLAGS, u8);
}

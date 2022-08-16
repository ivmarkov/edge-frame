pub use role::*;

mod role {
    use enumset::*;

    use serde::{Deserialize, Serialize};

    use strum_macros::{Display, EnumIter, EnumMessage, EnumString};

    use num_enum::TryFromPrimitive;

    #[derive(
        EnumSetType,
        Debug,
        PartialOrd,
        Serialize,
        Deserialize,
        EnumString,
        Display,
        EnumMessage,
        EnumIter,
        TryFromPrimitive,
    )]
    #[repr(u8)]
    pub enum Role {
        #[strum(serialize = "none", message = "None")]
        None,

        #[strum(serialize = "user", message = "User")]
        User,

        #[strum(serialize = "admin", message = "Admin")]
        Admin,
    }

    impl Default for Role {
        fn default() -> Self {
            Role::None
        }
    }
}

#[macro_export]
macro_rules! security_txt {
    ($name:expr) => {
        $crate::solana_security_txt::security_txt! {
            name: $name,
            project_url: "https://gmxsol.io",
            contacts: "email:security@gmxsol.io,email:admin@zenith.security,link:https://discord.gg/gmxsol",
            policy: "https://github.com/gmsol-labs/gmx-solana/blob/main/SECURITY.md",
            preferred_languages: "en",
            source_code: "https://github.com/gmsol-labs/gmx-solana",
            auditors: "Zenith,Sherlock"
        }
    };
}

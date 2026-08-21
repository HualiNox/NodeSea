macro_rules! define_settings {
    ($name:ident, $doc:literal, $base:ident; $( $variant:ident => $lib_name:literal = $offset:literal ),+ $(,)?) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum $name {
            $(
                #[doc = concat!("The `", $lib_name, "` setting.")]
                $variant = $base + $offset,
            )+
        }
    };
}

pub(super) use define_settings;

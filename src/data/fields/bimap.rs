macro_rules! create_bimap {
    ($idn:ident { $lt:ty => $rt:ty, $($l:expr => $r:expr,)* }) => {
        static $idn: once_cell::sync::Lazy<$crate::data::fields::bimap::BiMap<$lt, $rt>> = once_cell::sync::Lazy::new(|| {
            let mut map = $crate::data::fields::bimap::BiMap::new();
            $(map.insert($l, $r);)*
            map
        });
    }
}

pub(super) use bimap::BiMap;
pub(super) use create_bimap;

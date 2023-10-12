macro_rules! make_vec {
    ($struct:ident, $($name:ident$(,)*)+) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $struct {
            $(
                pub $name: f32,
            )+
        }

        impl $struct {
            #[allow(unused)]
            pub fn new($($name: f32,)+) -> $struct {
                $struct {
                    $(
                        $name,
                    )+
                }
            }

            #[allow(unused)]
            pub fn empty() -> $struct {
                $(
                    let $name = 0.0;
                )+
                $struct::new($($name,)+)
            }
        }
    };
}

make_vec!(Vec2, x, y);
make_vec!(Vec3, x, y, z);
make_vec!(Vec4, x, y, z, w);

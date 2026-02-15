macro_rules! handlers {
    ($($name: ident: $types: ty),+ $(,)?) => {
        #[derive(Debug, Clone)]
        pub struct Handlers {
            $(
                $name: EventHandler<$types>,
            )+
        }

        impl Handlers {
            pub(crate) fn new(
            $(
                $name: impl FnMut($types) -> Result<()> + 'static,
            )+
            ) -> Self {
                Self {
                    $(
                        $name: EventHandler::new($name),
                    )+
                     }
            }
        }
    };
}

pub(crate) use handlers;

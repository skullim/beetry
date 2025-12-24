mod models;
mod persistence;

pub use models::*;
pub use persistence::*;

#[macro_export]
macro_rules! spec {
    ( type = action,
      name = $name: literal
      $(, params = $params_schema:expr)?
      $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),* ])?
      $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
    ) =>
    {
        spec! {
            spec_kind = $crate::NodeKind::Action,
            name = $name
            $(, params = $params_schema)?
            $(, senders = [$($snd_ty, desc = $snd_desc),*])?
            $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
        }

    };

    ( type = condition,
      name = $name: literal
      $(, params = $params_schema:expr )?
      $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
      $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
    ) =>
    {
        spec! {
            spec_kind = $crate::NodeKind::Condition,
            name = $name
            $(, params = $params_schema)?
            $(, senders = [$($snd_ty, desc = $snd_desc),*])?
            $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
        }
    };

    (
        spec_kind = $spec_kind: expr,
        name = $name: literal
        $(, params = $params_schema:expr )?
        $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
        $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
    ) =>
     {
        {
            //@todo add creation of PortsSpec
            let builder = $crate::NodeSpec::builder().key($crate::NodeSpecKey::new($crate::NodeName::new($name), $spec_kind));
            $(
                let builder = builder.params_schema($params_schema);
            )?
            builder.build()
            // let builder = $builder.name($name);
            // builder.schema($crate::schema! {
            //     kind = $schema_kind
            //     $(, senders = [$($snd_ty, desc = $snd_desc),*])?
            //     $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
            // }).build()
        }
     }
}

#[macro_export]
macro_rules! schema {
    (   kind = action
        $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
        $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
    ) =>
    {
        $crate::schema! { schema_builder = $crate::node::ActionLeafSchema::builder()
                          $(, senders = [$($snd_ty, desc = $snd_desc),*])?
                          $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
        }
    };


    (   kind = condition
        $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
        $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
    ) =>
    {
        $crate::schema! { schema_builder = $crate::node::ConditionLeafSchema::builder()
                          $(, senders = [$($snd_ty, desc = $snd_desc),*])?
                          $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
        }
    };

    (   schema_builder = $builder: expr
        $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
        $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
    ) =>
    {
        {
            let builder = $builder;
            $(
                let builder = builder.senders([
                    $(
                        $crate::channel::MessageSpec::new::<$snd_ty>($snd_desc),
                    )*
                ]);
            )?
            $(
                let builder = builder.receivers([
                    $(
                        $crate::channel::MessageSpec::new::<$rcv_ty>($rcv_desc),
                    )*
                ]);
            )?
            builder.build()
        }
    }
}

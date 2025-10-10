use std::marker::PhantomData;

use uuid::Uuid;

use crate::{Reflectionable, TypedRx, TypedTx};

pub struct TransportBuilder<Req: Reflectionable, Res: Reflectionable> {
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: Reflectionable, Res: Reflectionable> TransportBuilder<Req, Res> {
    fn server(uuid: Uuid) -> TransportServerBuilder<Req, Res> {
        TransportServerBuilder::new(uuid)
    }
}

pub struct TransportServerBuilder<Req: Reflectionable, Res: Reflectionable> {
    uuid: Uuid,
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: Reflectionable, Res: Reflectionable> TransportServerBuilder<Req, Res> {
    pub fn new(uuid: Uuid) -> TransportServerBuilder<Req, Res> {
        TransportServerBuilder {
            uuid: uuid,
            _phantom: PhantomData,
        }
    }

    pub fn build() -> (TypedTx<Req>, TypedRx<Res>) {
        return TypedTx::new(inner);
    }
}

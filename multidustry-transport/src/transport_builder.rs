use std::marker::PhantomData;

use uuid::Uuid;

use crate::{Needed, TypedRx, TypedTx};

pub struct TransportBuilder<Req: Needed, Res: Needed> {
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: Needed, Res: Needed> TransportBuilder<Req, Res> {
    fn server(uuid: Uuid) -> TransportServerBuilder<Req, Res> {
        TransportServerBuilder::new(uuid)
    }
}

pub struct TransportServerBuilder<Req: Needed, Res: Needed> {
    uuid: Uuid,
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: Needed, Res: Needed> TransportServerBuilder<Req, Res> {
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

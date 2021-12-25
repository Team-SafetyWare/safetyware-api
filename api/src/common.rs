pub trait HasId {
    type Id;
}

pub trait GetId: HasId {
    fn id(&self) -> Self::Id;
}

pub trait SetId: HasId {
    fn set_id(&mut self, id: Self::Id);
}

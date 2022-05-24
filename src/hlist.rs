pub trait HList {}

#[derive(Default)]
pub struct Nil;
impl HList for Nil {}
#[derive(Default)]
pub struct Cons<T, Tail>(T, Tail);
impl<T, Tail: HList> HList for Cons<T, Tail> {}

pub trait HListExt: Sized {
    fn push<T>(self, t: T) -> Cons<T, Self>;
}

impl<H: HList> HListExt for H {
    fn push<T>(self, t: T) -> Cons<T, Self> {
        Cons(t, self)
    }
}

pub trait Index {}
pub struct Z;
pub struct Succ<T>(T);
impl Index for Z {}
impl<T: Index> Index for Succ<T> {}

pub trait ContainsAt<T, I> {
    fn take(&self) -> &T;
    fn take_mut(&mut self) -> &mut T;
}

impl<T, Tail: HList> ContainsAt<T, Z> for Cons<T, Tail> {
    fn take(&self) -> &T {
        &self.0
    }

    fn take_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
impl<T, H, Tail: HList, I: Index> ContainsAt<T, Succ<I>> for Cons<H, Tail>
where
    Tail: ContainsAt<T, I>,
{
    fn take(&self) -> &T {
        self.1.take()
    }

    fn take_mut(&mut self) -> &mut T {
        self.1.take_mut()
    }
}

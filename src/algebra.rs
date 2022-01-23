
pub trait Swizzling2<T: Copy + Clone>{
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn xy(&self) -> Self;
    fn yx(&self) -> Self;
}

pub trait Swizzling3<T: Copy + Clone>{
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;
    fn xy<Sw2: Swizzling2<T>>(&self) -> Sw2;
    fn yx<Sw2: Swizzling2<T>>(&self) -> Sw2;
    fn xz<Sw2: Swizzling2<T>>(&self) -> Sw2;
    fn zx<Sw2: Swizzling2<T>>(&self) -> Sw2;
    fn yz<Sw2: Swizzling2<T>>(&self) -> Sw2;
    fn zy<Sw2: Swizzling2<T>>(&self) -> Sw2;
    fn xyz(&self) -> Self;
    fn xzy(&self) -> Self;
    fn yxz(&self) -> Self;
    fn yzx(&self) -> Self;
    fn zxy(&self) -> Self;
    fn zyx(&self) -> Self;
}

impl<T: Copy + Clone> Swizzling2<T> for [T; 2]{
    fn x(&self) -> T {
        self[0]
    }

    fn y(&self) -> T {
        self[1]
    }

    fn xy(&self) -> Self {
        *self
    }

    fn yx(&self) -> Self {
        [self[1], self[0]]
    }
}

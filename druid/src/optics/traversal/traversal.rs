use std::marker::PhantomData;

pub trait Traversal<T1: ?Sized, T2: ?Sized> {
    fn with<'data, V, F>(&'data self, data: &'data T1, f: F) -> Box<dyn Iterator<Item = V> + 'data>
    where
        T2: 'data,
        V: 'data,
        F: FnOnce(&'data T2) -> V + 'data + Copy;
    fn with_mut<'data, V, F>(
        &'data self,
        data: &'data mut T1,
        f: F,
    ) -> Box<dyn Iterator<Item = V> + 'data>
    where
        T2: 'data,
        V: 'data,
        F: FnOnce(&'data mut T2) -> V + 'data + Copy;
}

pub struct VecTraversal;

impl<T2> Traversal<Vec<T2>, T2> for VecTraversal {
    fn with<'data, V, F>(
        &'data self,
        data: &'data Vec<T2>,
        f: F,
    ) -> Box<dyn Iterator<Item = V> + 'data>
    where
        T2: 'data,
        V: 'data,
        F: FnOnce(&'data T2) -> V + 'data + Copy,
    {
        Box::new(data.iter().map(move |t2| f(t2)))
    }
    fn with_mut<'data, V, F>(
        &'data self,
        data: &'data mut Vec<T2>,
        f: F,
    ) -> Box<dyn Iterator<Item = V> + 'data>
    where
        T2: 'data,
        V: 'data,
        F: FnOnce(&'data mut T2) -> V + 'data + Copy,
    {
        Box::new(data.iter_mut().map(move |t2| f(t2)))
    }
}

impl<Tr1, Tr2, T1, T2, T3> Traversal<T1, T3> for Then<Tr1, Tr2, T2>
where
    T1: ?Sized,
    T2: ?Sized,
    T3: ?Sized,
    Tr1: Traversal<T1, T2>,
    Tr2: Traversal<T2, T3>,
{
    fn with<'data, V, F>(&'data self, data: &'data T1, f: F) -> Box<dyn Iterator<Item = V> + 'data>
    where
        T3: 'data,
        V: 'data,
        F: FnOnce(&'data T3) -> V + 'data + Copy,
    {
        Box::new(
            self.left
                .with(data, move |b: &'data T2| self.right.with(b, f))
                .flatten(),
        )
    }
    fn with_mut<'data, V, F>(
        &'data self,
        data: &'data mut T1,
        f: F,
    ) -> Box<dyn Iterator<Item = V> + 'data>
    where
        T3: 'data,
        V: 'data,
        F: FnOnce(&'data mut T3) -> V + 'data + Copy,
    {
        Box::new(
            self.left
                .with_mut(data, move |b: &'data mut T2| self.right.with_mut(b, f))
                .flatten(),
        )
    }
}

#[derive(Debug, Copy, PartialEq)]
pub struct Then<Tr1, Tr2, T2: ?Sized> {
    left: Tr1,
    right: Tr2,
    _marker: PhantomData<T2>,
}

impl<Tr1, Tr2, T2: ?Sized> Then<Tr1, Tr2, T2> {
    pub fn new<T1: ?Sized, T3: ?Sized>(left: Tr1, right: Tr2) -> Self
    where
        Tr1: Traversal<T1, T2>,
        Tr2: Traversal<T2, T3>,
    {
        Self {
            left,
            right,
            _marker: PhantomData,
        }
    }
}

impl<Tr1: Clone, Tr2: Clone, T2> Clone for Then<Tr1, Tr2, T2> {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            _marker: PhantomData,
        }
    }
}

// cargo test optics::traversal::traversal::test_vec_traversal_with -- --exact
#[test]
fn test_vec_traversal_with() {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn u8_t2_plus_1(t2: &u8) -> u8 {
        *t2 + 1
    }

    let v = vec![0u8, 1, 2];
    let res = VecTraversal.with(&v, u8_t2_plus_1);
    let res: Vec<_> = res.collect();
    assert_eq!(res, vec![1, 2, 3]);

    let v2 = vec![vec![0, 1, 2], vec![10, 11, 12]];
    let trav = Then::new(VecTraversal, VecTraversal);
    let res2 = trav.with(&v2, u8_t2_plus_1);
    let res2: Vec<_> = res2.collect();
    assert_eq!(res2, vec![1, 2, 3, 11, 12, 13]);
}

// cargo test optics::traversal::traversal::test_vec_traversal_with_mut -- --exact
#[test]
fn test_vec_traversal_with_mut() {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn u8_t2_plus_1_mut(t2: &mut u8) {
        *t2 += 1;
    }

    let mut v = vec![0u8, 1, 2];
    let res = VecTraversal.with_mut(&mut v, u8_t2_plus_1_mut);
    // needs to run the iterator to apply changes
    let () = res.take(1).collect(); // so only applies for the first
    assert_eq!(
        v,
        vec![
            // changed value, from 0 to 1
            1, //
            // old values
            1, 2
        ]
    );

    let mut v2 = vec![vec![0, 1, 2], vec![10, 11, 12]];
    let trav = Then::new(VecTraversal, VecTraversal);
    let res2 = trav.with_mut(&mut v2, u8_t2_plus_1_mut);
    // needs to run the iterator to apply changes
    // so only apply the change for the first 4 values
    let () = res2.take(4).collect();
    assert_eq!(
        v2,
        // changed values
        vec![
            vec![1, 2, 3],
            vec![
                11, // still changed value
                // old values
                11, 12
            ],
        ]
    );
}

use super::*;

impl private::IndexGetDataRefForGetRef<usize> for CowList {
    fn get_ref(&self, key: &usize) -> Option<&data::Data> {
        self.get(*key)
    }
}

impl private::IndexGetDataRefForGet<usize> for CowList {
    fn get_ref(&self, key: &usize) -> Option<&data::Data> {
        self.get(*key)
    }
}

impl private::IndexSetData<usize> for CowList {
    fn set(&mut self, key: usize, data: data::Data) {
        self.set(key, data);
    }
}

impl private::IndexGetDataMut<usize> for CowList {
    fn get_mut(&mut self, key: &usize) -> Option<&mut data::Data> {
        self.get_mut(*key)
    }
}

impl<T> VectorOpr<T> for CowList
where
    T: From<data::Data>,
    data::Data: From<T>,
{
    fn xpop_front(&mut self) -> Option<T> {
        self.pop_front().map(T::from)
    }

    fn xpop_back(&mut self) -> Option<T> {
        self.pop_back().map(T::from)
    }

    fn xpush_back(&mut self, value: T) {
        self.push_back(data::Data::from(value))
    }

    fn xpush_front(&mut self, value: T) {
        self.push_front(data::Data::from(value))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basis() {
        let mut list = CowList::new();
        list.xpush_back(1f64);
        list.xpush_back(1i64);
        list.xpush_back(false);
        assert_eq!(list.len(), 3);
        assert_eq!(list.xget(&1), Some(1i64));
        assert_eq!(list.xpop_back(), Some(false));
    }
}

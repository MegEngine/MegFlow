use super::*;

impl<Output> IndexGet<usize, Output> for List
where
    Output: Copy,
    data::Data: data::IndexGet<usize, Output>,
{
    fn xget(&self, i: &usize) -> Option<Output> {
        data::IndexGet::get(self, *i)
    }
}

impl<Value> IndexSet<usize, Value> for List
where
    Value: Copy,
    data::Data: data::IndexSet<usize, Value>,
{
    fn xset(&mut self, i: usize, data: Value) {
        data::IndexSet::set(self, i, data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basis() {
        let list = List::from(vec![0i64, 1, 2, 3, 4]);
        assert_eq!(list.len(), 5);
        assert_eq!(list.xget(&3), Some(3i64));
        let list: Vec<i64> = list.into();
        assert_eq!(list, vec![0i64, 1, 2, 3, 4]);
    }
}

use ink::storage::Mapping;

pub type Set<T> = Mapping<T, ()>;

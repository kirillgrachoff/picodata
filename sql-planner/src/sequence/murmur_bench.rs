// use criterion::{Criterion, criterion_group, criterion_main};
use tarantool::tuple::{FieldType, KeyDef, KeyDefPart, Tuple};

#[test]
pub fn murmur3() {
    let part = KeyDefPart {
        field_no: 0,
        field_type: FieldType::Unsigned,
        is_nullable: false,
        collation: None,
        path: None,
    };

    let key_def = KeyDef::new(&[part]).expect("failed to create key_def");

    let tuple_a = Tuple::new(&(10_u64, "hello", 42)).expect("cannot create tuple");
    let tuple_b = Tuple::new(&(10_u64, "world", 754)).expect("cannot create tuple");

    let tuple_c = Tuple::new(&(42_u64, "fizzbuzz", 11)).expect("cannot create tuple");

    let hash_a = key_def.hash(&tuple_a);
    let hash_b = key_def.hash(&tuple_b);
    let hash_c = key_def.hash(&tuple_c);

    assert_eq!(hash_a, hash_b);
    assert_ne!(hash_a, hash_c);

    println!("hash: {}, {}, {}", hash_a, hash_b, hash_c);
}

// fn bench_murmur3(c: &mut Criterion) {
//     let mut group = c.benchmark_group("MurMur3");

//     let part = KeyDefPart {
//         field_no: 0,
//         field_type: FieldType::Unsigned,
//         is_nullable: false,
//         collation: None,
//         path: None,
//     };

//     let key_def = KeyDef::new(&[part]).expect("failed to create key_def");

//     let tuple_a = Tuple::new(&(10_u64, "hello", 42)).expect("cannot create tuple");
//     let tuple_b = Tuple::new(&(10_u64, "world", 754)).expect("cannot create tuple");

//     let tuple_c = Tuple::new(&(42_u64, "fizzbuzz", 11)).expect("cannot create tuple");

//     let hash_a = key_def.hash(&tuple_a);
//     let hash_b = key_def.hash(&tuple_b);
//     let hash_c = key_def.hash(&tuple_c);

//     assert_eq!(hash_a, hash_b);
//     assert_ne!(hash_a, hash_c);


//     group.bench_function("HashRate", |b| {
//         b.iter(|| {
//             let t = Tuple::new(&10_u64).expect("cannot construct");
//         });
//     });
// }

// criterion_group!(benches, bench_murmur3);
// criterion_main!(benches);
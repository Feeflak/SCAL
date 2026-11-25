use scal_core::prelude::*;
#[test]
fn t() {
    let v = timeline![
        wait(1.0.s())
        wait(2.0.s()),
        parallel![
            wait(1.0.s())
            sequence![wait(1.0.s()) wait(2.0.s())]
            if true { wait(1.0.s()) } else { wait(2.0.s()) }
            wait(3.0.s())
        ],
        wait(5.0.s()),
    ];
    assert_eq!(v.len(), 4);
    match &v[2] {
        AnimOP::All(inner, _) => assert_eq!(inner.len(), 4),
        _ => panic!(),
    }
}

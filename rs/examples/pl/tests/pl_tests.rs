use async_lock::RwLock;
use pl::{
    add_assign, block, call, define, float64, function, mul, mul_assign, plum_ref, sub, sub_assign,
    symbolic_ref, ASTNode, Expr, Runtime,
};
use std::sync::Arc;

/// This will run once at load time (i.e. presumably before main function is called).
#[ctor::ctor]
fn overall_init() {
    env_logger::init();
}

pub struct TestData {
    pub norm_squared_function: ASTNode,
    pub exp_function: ASTNode,
    pub cos_function: ASTNode,
    pub sin_function: ASTNode,
}

impl TestData {
    pub fn new() -> Self {
        // Define some functions (which will be used in several places)
        let norm_squared_function = function!((x, y) -> block! {
            define!(z, symbolic_ref!(x) * symbolic_ref!(x));
            add_assign!(z, symbolic_ref!(y) * symbolic_ref!(y));
            ;
            symbolic_ref!(z)
        });
        let exp_function = function!((x) -> block! {
            define!(z, symbolic_ref!(x));
            define!(y, float64!(1.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(2.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(3.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(4.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(5.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(6.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(7.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(8.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(9.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(10.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(11.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(12.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(13.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(14.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(15.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(16.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(17.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(18.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(19.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(20.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(21.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(22.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(23.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(24.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(25.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(26.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x) / float64!(27.0));
            add_assign!(y, symbolic_ref!(z));
            ;
            symbolic_ref!(y)
        });
        let cos_function = function!((x) -> block! {
            define!(x_squared, symbolic_ref!(x) * symbolic_ref!(x));
            define!(z, symbolic_ref!(x_squared) / float64!(1.0*2.0));
            define!(y, float64!(1.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(3.0*4.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(5.0*6.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(7.0*8.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(9.0*10.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(11.0*12.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(13.0*14.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(15.0*16.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(17.0*18.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(19.0*20.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(21.0*22.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(23.0*24.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(25.0*26.0));
            ;
            symbolic_ref!(y)
        });
        let sin_function = function!((x) -> block! {
            define!(x_squared, mul!(symbolic_ref!(x), symbolic_ref!(x)));
            define!(z, symbolic_ref!(x));
            define!(y, symbolic_ref!(x));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(2.0*3.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(4.0*5.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(6.0*7.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(8.0*9.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(10.0*11.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(12.0*13.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(14.0*15.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(16.0*17.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(18.0*19.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(20.0*21.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(22.0*23.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(24.0*25.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(26.0*27.0));
            ;
            symbolic_ref!(y)
        });

        Self {
            norm_squared_function,
            exp_function,
            cos_function,
            sin_function,
        }
    }
}

#[tokio::test]
async fn test_eval() {
    let mut rt = Runtime::new();

    let expr = float64!(123.456);
    let value = expr
        .eval(&mut rt)
        .await
        .expect("pass")
        .as_float64()
        .expect("pass")
        .as_f64();
    log::debug!("expr -> {:.17}", value);
    assert_eq!(value, 123.456);
}

#[tokio::test]
async fn test_arithmetic() {
    let mut rt = Runtime::new();

    log::debug!("-- cos(1) using arithmetic expression macros --------------");
    // cos(1) ~= 1 - 1/2*(1 - 1/(3*4)*(1 - 1/(5*6)*(1 - 1/(7*8)*(1 - 1/(9*10)))))
    //         = 0.54030230379188715
    // is correct to 7 digits; actual value is 0.5403023058681398...
    let cos_1 = sub!(
        float64!(1.0),
        mul!(
            float64!(1.0 / 2.0),
            sub!(
                float64!(1.0),
                mul!(
                    float64!(1.0 / (3.0 * 4.0)),
                    sub!(
                        float64!(1.0),
                        mul!(
                            float64!(1.0 / (5.0 * 6.0)),
                            sub!(
                                float64!(1.0),
                                mul!(
                                    float64!(1.0 / (7.0 * 8.0)),
                                    sub!(float64!(1.0), float64!(1.0 / (9.0 * 10.0)),)
                                )
                            )
                        )
                    )
                )
            )
        )
    );
    let value = cos_1
        .eval(&mut rt)
        .await
        .expect("pass")
        .as_float64()
        .expect("pass")
        .as_f64();
    log::debug!("cos_1 -> {:.17}", value);
    assert!((value - 1.0f64.cos()) < 1.0e-7);
}

#[tokio::test]
async fn test_ops_syntax() {
    let mut rt = Runtime::new();

    log::debug!("-- cos(1) using std::ops arithmetic traits for better syntax --------------");
    let one = float64!(1.0);
    let cos_1 = one.clone()
        - float64!(1.0 / 2.0)
            * (one.clone()
                - float64!(1.0 / (3.0 * 4.0))
                    * (one.clone()
                        - float64!(1.0 / (5.0 * 6.0))
                            * (one.clone()
                                - float64!(1.0 / (7.0 * 8.0))
                                    * (one.clone() - float64!(1.0 / (9.0 * 10.0))))));
    let value = cos_1
        .eval(&mut rt)
        .await
        .expect("pass")
        .as_float64()
        .expect("pass")
        .as_f64();
    log::debug!("cos_1 -> {:.17}", value);
    assert!((value - 1.0f64.cos()) < 1.0e-7);
}

#[tokio::test]
async fn test_block_0() {
    let mut rt = Runtime::new();

    log::debug!("-- block_0 --------------");
    let block_0 = block!(float64!(1.2));
    let value = block_0
        .eval(&mut rt)
        .await
        .expect("pass")
        .as_float64()
        .expect("pass")
        .as_f64();
    log::debug!("block_0 -> {:.17}", value);
    assert_eq!(value, 1.2);
}

#[tokio::test]
async fn test_block_1() {
    let mut rt = Runtime::new();

    let block_1 = block! {
        define!(x, float64!(303.404));
        define!(y, float64!(888.9999));;
        symbolic_ref!(x)
    };
    let value = block_1
        .eval(&mut rt)
        .await
        .expect("pass")
        .as_float64()
        .expect("pass")
        .as_f64();
    log::debug!("block_1 -> {:.17}", value);
    assert_eq!(value, 303.404);
}

#[tokio::test]
async fn test_symbolic_ref() {
    let mut rt = Runtime::new();

    // Compute cos(2)
    let cos_2 = block! {
        define!(x, float64!(2.0));
        define!(x_squared, symbolic_ref!(x) * symbolic_ref!(x));
        define!(z, float64!(1.0/(1.0*2.0)) * symbolic_ref!(x_squared));
        define!(y, float64!(1.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(3.0*4.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(5.0*6.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(7.0*8.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(9.0*10.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(11.0*12.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(13.0*14.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(15.0*16.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(17.0*18.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(19.0*20.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(21.0*22.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(23.0*24.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(25.0*26.0));
        ;
        symbolic_ref!(y)
    };
    let value = cos_2
        .eval(&mut rt)
        .await
        .expect("pass")
        .as_float64()
        .expect("pass")
        .as_f64();
    let error = (value - 2.0f64.cos()).abs();
    log::debug!(
        "cos_2 -> {:.17}; 'actual' value is {:.17}; error: {:.17e}",
        value,
        2.0f64.cos(),
        error,
    );
    assert!(error < 1.0e-16);
}

#[tokio::test]
async fn test_call() {
    let mut rt = Runtime::new();

    let test_data = TestData::new();

    log::debug!("-- exp function ----------------------------------------------------");
    let program = block! {
        define!(norm_squared, test_data.norm_squared_function.clone());
        define!(exp, test_data.exp_function.clone());
        define!(cos, test_data.cos_function.clone());
        define!(sin, test_data.sin_function.clone());
        ;
        // call!(symbolic_ref!(norm_squared), (call!(symbolic_ref!(cos), (float64!(0.83))), call!(symbolic_ref!(sin), (float64!(0.83)))))
        call!(symbolic_ref!(exp), (float64!(1.0)))
    }
    .into_block()
    .expect("pass");
    let program_return_value = program
        .run_as_program(&mut rt)
        .await
        .expect("pass")
        .as_float64()
        .expect("pass")
        .as_f64();
    let expected_value = 1.0f64.exp();
    let error = (program_return_value - expected_value).abs();
    log::debug!(
        "program -> {:.17}; expected value: {:.17}; error: {:.17e}",
        program_return_value,
        expected_value,
        error,
    );
    assert!(error < 5.0e-16);
    log::debug!("rt.symbol_mv.len(): {}", rt.symbol_mv.len());
    log::debug!(
        "rt.symbol_mv.last().unwrap().keys():\n{:#?}",
        rt.symbol_mv.last().unwrap().keys()
    );
}

#[tokio::test]
async fn test_plum_ref_call() {
    let mut rt = Runtime::new();

    // Now create a Datahost and initialize the Datacache with it.
    let datahost_la = Arc::new(RwLock::new(idp_core::Datahost::open(
        idp_datahost_storage_sqlite::DatahostStorageSQLite::new_in_memory()
            .await
            .expect("pass"),
    )));
    idp_core::Datacache::set_singleton(Box::new(idp_core::Datacache::new(datahost_la.clone())));

    let test_data = TestData::new();

    // Store the functions in the Datahost
    let norm_squared_plum_head_seal = datahost_la
        .write()
        .await
        .store_plum(
            &idp_proto::PlumBuilder::new()
                .with_plum_body_content_type(idp_proto::ContentType::from(
                    "pl/function".as_bytes().to_vec(),
                ))
                .with_plum_body_content(
                    rmp_serde::to_vec(&test_data.norm_squared_function).expect("pass"),
                )
                .build()
                .expect("pass"),
        )
        .await
        .expect("pass");
    let exp_plum_head_seal = datahost_la
        .write()
        .await
        .store_plum(
            &idp_proto::PlumBuilder::new()
                .with_plum_body_content_type(idp_proto::ContentType::from(
                    "pl/function".as_bytes().to_vec(),
                ))
                .with_plum_body_content(rmp_serde::to_vec(&test_data.exp_function).expect("pass"))
                .build()
                .expect("pass"),
        )
        .await
        .expect("pass");
    let cos_plum_head_seal = datahost_la
        .write()
        .await
        .store_plum(
            &idp_proto::PlumBuilder::new()
                .with_plum_body_content_type(idp_proto::ContentType::from(
                    "pl/function".as_bytes().to_vec(),
                ))
                .with_plum_body_content(rmp_serde::to_vec(&test_data.cos_function).expect("pass"))
                .build()
                .expect("pass"),
        )
        .await
        .expect("pass");
    let sin_plum_head_seal = datahost_la
        .write()
        .await
        .store_plum(
            &idp_proto::PlumBuilder::new()
                .with_plum_body_content_type(idp_proto::ContentType::from(
                    "pl/function".as_bytes().to_vec(),
                ))
                .with_plum_body_content(rmp_serde::to_vec(&test_data.sin_function).expect("pass"))
                .build()
                .expect("pass"),
        )
        .await
        .expect("pass");

    // Here, create a program where the functions are linked in from hash-addressed Plums in the Datahost,
    // automatically loaded via PlumRef and Datacache.
    log::debug!("- Program with functions linked via PlumRef -----------------------------------------------------");
    let program = block! {
        define!(norm_squared, plum_ref!(norm_squared_plum_head_seal.clone().into()));
        define!(exp, plum_ref!(exp_plum_head_seal.clone().into()));
        define!(cos, plum_ref!(cos_plum_head_seal.clone().into()));
        define!(sin, plum_ref!(sin_plum_head_seal.clone().into()));
        ;
        call!(symbolic_ref!(norm_squared), (call!(symbolic_ref!(cos), (float64!(0.83))), call!(symbolic_ref!(sin), (float64!(0.83)))))
        // call!(symbolic_ref!(exp), (float64!(1.0)))
    }
    .into_block().expect("pass");
    let program_return_value = program
        .run_as_program(&mut rt)
        .await
        .expect("pass")
        .as_float64()
        .expect("pass")
        .as_f64();
    log::debug!("program_return_value: {:.17}", program_return_value);
    assert!((program_return_value - 1.0).abs() < 1.0e-10);
    log::debug!("rt.symbol_mv.len(): {}", rt.symbol_mv.len());
    log::debug!(
        "rt.symbol_mv.last().unwrap().keys():\n{:#?}",
        rt.symbol_mv.last().unwrap().keys()
    );
}

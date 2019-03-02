/* use super::compiler;
use super::cbms; */

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

/* #[test]
fn test_compiler_1() {
    let raw_bms = "#00103:100D02\n#00203:110h20";
    let cbms = compiler::import_bms(raw_bms)
        .expect("An error has occured during BMS compilation: ");
    let expected_cbms = cbms::CBMS {
        commands: vec![
            cbms::CBMSCommand::Channel {
                measure: 1, 
                channel_id: 3,
                args_beg: 0,
                args_cnt: 3,
            },
            cbms::CBMSCommand::Channel {
                measure: 2,
                channel_id: 3,
                args_beg: 3,
                args_cnt: 3,
            },
        ],
        channel_args: vec![36, 13, 2, 37, 17, 72],
    };
    assert_eq!(cbms, expected_cbms);
} */
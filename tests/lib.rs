// extern crate yasl_compiler;
//
// use yasl_compiler::{compile_file, ParserResult};
//
// macro_rules! test_file {
//     ($file_name:expr) => (
//         let r = compile_file(format!("p6/{}", $file_name));
//
//         // Check output against the expected output
//
//         match r {
//             ParserResult::Success => {},
//             _ => panic!(),
//         }
//     );
// }
//
// /* ================================
//  * ========== Pass Cases ==========
//  * ================================ */
//
// #[test]
// fn pass1() {
//     test_file!("pass1.txt");
// }
//
// #[test]
// fn pass2() {
//     test_file!("pass2.txt");
// }
//
// #[test]
// fn pass3() {
//     test_file!("pass3.txt");
// }
//
// #[test]
// fn pass4() {
//     test_file!("pass4.txt");
// }
//
// #[test]
// fn pass5() {
//     test_file!("pass5.txt");
// }
//
// #[test]
// fn pass6() {
//     test_file!("pass6.txt");
// }
//
// #[test]
// fn pass7() {
//     test_file!("pass7.txt");
// }
//
// #[test]
// fn pass8() {
//     test_file!("pass8.txt");
// }
//
// #[test]
// fn pass9() {
//     test_file!("pass9.txt");
// }
//
// #[test]
// fn pass10() {
//     test_file!("pass10.txt");
// }
//
// // /* ================================
// //  * ========== Fail Cases ==========
// //    ================================ */
// //
// // #[test]
// // #[should_panic]
// // fn fail1() {
// //     test_file!("fail1.txt");
// // }
// //
// // #[test]
// // #[should_panic]
// // fn fail2() {
// //     test_file!("fail2.txt");
// // }
// //
// // #[test]
// // #[should_panic]
// // fn fail3() {
// //     test_file!("fail3.txt");
// // }
// //
// // #[test]
// // #[should_panic]
// // fn fail4() {
// //     test_file!("fail4.txt");
// // }
// //
// // #[test]
// // #[should_panic]
// // fn fail5() {
// //     test_file!("fail5.txt");
// // }
// //
// // #[test]
// // #[should_panic]
// // fn fail6() {
// //     test_file!("fail6.txt");
// // }
// //
// // #[test]
// // #[should_panic]
// // fn fail7() {
// //     test_file!("fail7.txt");
// // }
// //
// // #[test]
// // #[should_panic]
// // fn fail8() {
// //     test_file!("fail8.txt");
// // }
// //
// // #[test]
// // #[should_panic]
// // fn fail9() {
// //     test_file!("fail9.txt");
// // }
// //
// // #[test]
// // #[should_panic]
// // fn fail10() {
// //     test_file!("fail10.txt");
// // }

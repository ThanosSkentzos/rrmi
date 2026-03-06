#[cfg(test)]
mod tests {
    use core::panic;

    use crate::{remote::RMI_ID, transport::RMIRequest};
    #[test]
    fn serde_int() {
        let data: i32 = 1;
        let data_serial = serde_cbor::to_vec(&data).expect("int is serializable");
        eprintln!("data: {:?}", data);
        eprintln!("serialized: {:?}", data_serial);
        let result: i32 =
            serde_cbor::from_slice(&data_serial).expect("should be able to deserialize");
        eprintln!("deserialized: {:?}", result);

        assert_eq!(data, result);
    }

    #[test]
    fn serde_RMIRequest() {
        let object_id: RMI_ID = 1;
        let method_handler: String = String::from("this is a test");
        let serialized_args = vec![0, 1, 2];
        let data = RMIRequest::new(object_id, method_handler, serialized_args);
        let data_serial = serde_cbor::to_vec(&data).expect("int is serializable");
        eprintln!("data : {:?}", data);
        // eprintln!("serialized: {:?}",data_serial);
        let result: RMIRequest =
            serde_cbor::from_slice(&data_serial).expect("should be able to deserialize");
        eprintln!("after: {:?}", result);
        assert_eq!(data, result);
    }
    #[test]
    fn serde_args() {
        let object_id: RMI_ID = 1;
        let method_handler: String = String::from("this is a test");
        let arr = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let hostname: &str = "127.0.0.1";

        let combined =
            serde_cbor::to_vec(&(object_id, method_handler.clone(), arr.clone(), hostname))
                .unwrap();
        let (object_id_d, method_handler_d, arr_d, hostname_d): (RMI_ID, String, Vec<i32>, String) =
            serde_cbor::from_slice(&combined).expect("should work");
        assert_eq!(object_id, object_id_d);
        assert_eq!(method_handler, method_handler_d);
        assert_eq!(arr, arr_d);
        assert_eq!(hostname, hostname_d);
        eprintln!("{object_id_d},{method_handler_d},{arr_d:?},{hostname_d}")
    }
}

use crate::error::RMIError;
use crate::remote::RMIResult;
pub use serde::{Deserialize, Serialize};

pub fn marshal<T: Serialize>(data: &T) -> RMIResult<Vec<u8>> {
    serde_cbor::to_vec(&data).map_err(|e| {
        eprintln!("Marshaling error: {e}");
        RMIError::SerializationError(e.to_string())
    })
}
pub fn unmarshal<T: for<'de> Deserialize<'de>>(bytes: &Vec<u8>) -> RMIResult<T> {
    serde_cbor::from_slice(&bytes).map_err(|e| {
        eprintln!("Unmarshaling error: {e} on bytes: {bytes:?}");
        RMIError::DeserializationError(e.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::{marshal, unmarshal};
    use crate::{remote::RMI_ID, transport::RMIRequest};

    #[test]
    fn serde_int() {
        let data: i32 = 1;
        let data_serial = marshal(&data).expect("int is serializable");
        eprintln!("data: {:?}", data);
        eprintln!("serialized: {:?}", data_serial);
        let result: i32 = unmarshal(&data_serial).expect("bytes from int are deserializable");
        eprintln!("deserialized: {:?}", result);

        assert_eq!(data, result);
    }

    #[test]
    #[allow(non_snake_case)]
    fn serde_RMIRequest() {
        let object_id: RMI_ID = 1;
        let method_name: String = String::from("this is a test");
        let serialized_args = vec![0, 1, 2];
        let data = RMIRequest {
            object_id,
            method_name,
            serialized_args,
        };
        let data_serial = marshal(&data).expect("data are serializable");
        eprintln!("data : {:?}", data);
        // eprintln!("serialized: {:?}",data_serial);
        let result: RMIRequest = unmarshal(&data_serial).expect("bytes should be deserializable");
        eprintln!("after: {:?}", result);
        assert_eq!(data, result);
    }

    #[test]
    fn serde_args() {
        let object_id: RMI_ID = 1;
        let method_handler: String = String::from("this is a test");
        let arr = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let hostname: &str = "127.0.0.1";

        let combined = marshal(&(object_id, method_handler.clone(), arr.clone(), hostname))
            .expect("data are serializable");
        let (object_id_d, method_handler_d, arr_d, hostname_d): (RMI_ID, String, Vec<i32>, String) =
            unmarshal(&combined).expect("bytes should be deserializable");
        assert_eq!(object_id, object_id_d);
        assert_eq!(method_handler, method_handler_d);
        assert_eq!(arr, arr_d);
        assert_eq!(hostname, hostname_d);
        eprintln!("{object_id_d},{method_handler_d},{arr_d:?},{hostname_d}")
    }
}

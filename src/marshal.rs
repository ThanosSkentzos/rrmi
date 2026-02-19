#[cfg(test)]
mod tests{
    use core::panic;

    use crate::{registry::RMI_ID, transport::RMIRequest};
    #[test]
    fn serde_int() {
        let data:i32 = 1;
        let data_serial = serde_cbor::to_vec(&data).expect("int is serializable");
        println!("data: {:?}",data);
        println!("serialized: {:?}",data_serial);
        let result:i32 = serde_cbor::from_slice(&data_serial).expect("should be able to deserialize");
        println!("deserialized: {:?}",result);

        assert_eq!(data,result);
    }

    #[test]
    fn serde_RMIRequest() {
        let object_id:RMI_ID = 1;
        let method_handler:String = String::from("this is a test");
        let serialized_args = vec![0,1,2];
        let data = RMIRequest::new(object_id, method_handler, serialized_args);
        let data_serial = serde_cbor::to_vec(&data).expect("int is serializable");
        println!("data : {:?}",data);
        // println!("serialized: {:?}",data_serial);
        let result:RMIRequest = serde_cbor::from_slice(&data_serial).expect("should be able to deserialize");
        println!("after: {:?}",result);
        assert_eq!(data,result);
    }
}
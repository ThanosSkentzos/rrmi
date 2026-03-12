use std::any::type_name_of_val;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::TcpClient;
use crate::remote::{RMIResult, RemoteRef};
use crate::stub::{marshal, unmarshal};
use crate::transport::{RMIRequest, RMIResponse, Transport};

pub trait RemoteTrait: Send + Sync {
    fn run_stub<T: for<'de> Deserialize<'de>, A: Serialize>(&self, arg: A) -> RMIResult<T>;
}

pub trait RemoteTraitTest: Send + Sync {
    fn run_stub_test<T: for<'de> Deserialize<'de> + Default, A: Serialize + Debug>(
        &self,
        arg: A,
    ) -> RMIResult<T>;
}
#[derive(Debug, Clone)]
pub struct Stub {
    remote: RemoteRef,
}

impl Stub {
    pub fn new(remote: RemoteRef) -> Self {
        Stub { remote }
    }

    pub fn from(remote: RemoteRef) -> Self {
        Stub { remote }
    }

    pub fn get_ref(self) -> RemoteRef {
        self.remote.clone()
    }
}

impl RemoteTrait for Stub {
    fn run_stub<R: for<'de> Deserialize<'de>, A: Serialize>(&self, arg: A) -> RMIResult<R> {
        let serialized_args = marshal(&arg)?;

        let req = RMIRequest {
            object_id: self.remote.id,
            method_name: "method_name".into(),
            serialized_args,
        };
        eprintln!("req: {req:?}");
        let server_addr = self.remote.addr;
        let transport = TcpClient::new(server_addr);
        let response: RMIResponse = transport.send(req)?;

        let bytes: Vec<u8> = response.result?;
        let tuple: R = unmarshal(&bytes)?;
        Ok(tuple)
    }
}

impl RemoteTraitTest for Stub {
    fn run_stub_test<R: for<'de> Deserialize<'de> + Default, A: Serialize + Debug>(
        &self,
        arg: A,
    ) -> RMIResult<R> {
        let t = type_name_of_val(&arg);
        let ret = R::default();
        let t_ret = type_name_of_val(&ret);
        eprintln!("args: {arg:?} of type: {t:?} -> return {t_ret:?}");
        RMIResult::Ok(ret)
    }
}

mod tests {
    use std::any::type_name_of_val;

    use crate::remote::RMI_ID;

    use super::*;

    #[test]
    fn different_stub_R_A() {
        let s1 = Stub::new(RemoteRef::example());
        let arg1 = 42;
        let arg2 = "test".to_string();
        let arg3 = ();
        let arg4 = RMIRequest::default();

        let res0: i32 = s1.run_stub_test(()).unwrap();
        let res1: () = s1.run_stub_test(0).unwrap();
        let res1: i32 = s1.run_stub_test(arg1).unwrap();
        let res2: String = s1.run_stub_test((arg1, arg2.clone())).unwrap();
        let res3: () = s1.run_stub_test(()).unwrap();
        let res4: RMIRequest = s1.run_stub_test(("this is a test")).unwrap();
        let res5: () = s1.run_stub_test(RMIRequest::default()).unwrap();
        let res6: (RMIRequest, RMI_ID, i32) =
            s1.run_stub_test((arg1, arg2.clone(), arg3, arg4)).unwrap();

        let t1 = type_name_of_val(&res1);
        let t2 = type_name_of_val(&res2);
        let t3 = type_name_of_val(&res3);
        let t4 = type_name_of_val(&res4);
        let t5 = type_name_of_val(&res5);
        let t6 = type_name_of_val(&res6);

        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
        assert_ne!(t3, t4);
        assert_ne!(t4, t5);
        assert_ne!(t5, t6);
    }
}
//test if 2 functions with different args can compile

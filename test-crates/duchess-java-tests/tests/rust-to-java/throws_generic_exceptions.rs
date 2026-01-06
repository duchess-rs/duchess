//@run

use duchess::prelude::*;
use java::lang::RuntimeException;

duchess::java_package! {
    package exceptions;
    
    public class GenericExceptionThrower<E extends java.lang.Exception> {
        public exceptions.GenericExceptionThrower(E);
        public void throwsGenericAndSimple() throws E, java.lang.RuntimeException;
        public static <T, E2 extends java.lang.Exception> void throwsGenericException(T, E2) throws E2;
        public static java.lang.RuntimeException returnsException();
    }
}


fn main() -> duchess::Result<()> {
    let e = exceptions::GenericExceptionThrower::<RuntimeException>::returns_exception().execute().unwrap().unwrap();
    let e = exceptions::GenericExceptionThrower::<RuntimeException>::throws_generic_exception::<java::lang::Object, RuntimeException>("test", &e).execute().unwrap_err();
    let duchess::Error::Thrown(e) = e else {
        panic!("Did not throw exception as expected");
    };
    let runtime = e.try_downcast::<RuntimeException>().execute().unwrap().unwrap();
    let thrower = exceptions::GenericExceptionThrower::<RuntimeException>::new(&runtime);
    let e2 = thrower.throws_generic_and_simple().execute().unwrap_err();
    Ok(())
}


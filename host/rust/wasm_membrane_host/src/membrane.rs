use std::sync::{Arc, RwLock, Weak};


use crate::error::Error;
use wasmer::{Module, Instance, WasmPtr, Array, WasmerEnv, imports, Function, RuntimeError};

pub static VERSION: i32 = 1;

pub struct WasmMembrane {
    pub instance: Instance,
    //host: Arc<RwLock<WasmHost>>,
}

impl WasmMembrane {

    pub fn init(&self)->Result<(),Error>
    {
        let mut pass = true;
        match self.instance.exports.get_memory("memory")
        {
            Ok(_) => {
                self.log("wasm", "verified: memory");
            }
            Err(_) => {
                self.log("wasm", "failed: memory. could not access wasm memory. (expecting the memory module named 'memory')");
                pass=false
            }
        }

        match self.instance.exports.get_native_function::<(),i32>("membrane_guest_version"){
            Ok(func) => {
                self.log("wasm", "verified: membrane_guest_version( ) -> i32");
                match func.call()
                {
                    Ok(version) => {
                        if version == VERSION
                        {
                            self.log("wasm", format!("passed: membrane_guest_version( ) -> i32 [USING VERSION {}]", version).as_str());
                        }
                        else {
                            self.log("wasm", format!("fail : membrane_guest_version( ) -> i32 [THIS HOST CANNOT WORK WITH VERSION {}]", version).as_str());
                            pass = false;
                        }
                    }
                    Err(error) => {
                        self.log("wasm", "fail : membrane_guest_version( ) -> i32 [CALL FAILED]");
                    }
                }
            }
            Err(_) => {
                self.log("wasm", "failed: membrane_guest_version( ) -> i32");
                pass=false
            }
        }


        match self.instance.exports.get_native_function::<i32,i32>("membrane_guest_alloc_buffer"){
            Ok(_) => {
                self.log("wasm", "verified: membrane_guest_alloc_buffer( i32 ) -> i32");
            }
            Err(_) => {
                self.log("wasm", "failed: membrane_guest_alloc_buffer( i32 ) -> i32");
                pass=false
            }
        }

        match self.instance.exports.get_native_function::<i32,WasmPtr<u8,Array>>("membrane_guest_get_buffer_ptr"){
            Ok(_) => {
                self.log("wasm", "verified: membrane_guest_get_buffer_ptr( i32 ) -> *const u8");
            }
            Err(_) => {
                self.log("wasm", "failed: membrane_guest_get_buffer_ptr( i32 ) -> *const u8");
                pass=false
            }
        }

        match self.instance.exports.get_native_function::<i32,i32>("membrane_guest_get_buffer_len"){
            Ok(_) => {
                self.log("wasm", "verified: membrane_guest_get_buffer_len( i32 ) -> i32");
            }
            Err(_) => {
                self.log("wasm", "failed: membrane_guest_get_buffer_len( i32 ) -> i32");
                pass=false
            }
        }
        match self.instance.exports.get_native_function::<i32,()>("membrane_guest_dealloc_buffer"){
            Ok(_) => {
                self.log("wasm", "verified: membrane_guest_dealloc_buffer( i32 )");
            }
            Err(_) => {
                self.log("wasm", "failed: membrane_guest_dealloc_buffer( i32 )");
                pass=false
            }
        }

        match self.instance.exports.get_native_function::<(),()>("membrane_guest_init"){

            Ok(func) => {
                self.log("wasm", "verified: membrane_guest_init()");

                match func.call()
                {
                    Ok(_) => {
                        self.log("wasm", "passed: membrane_guest_init()");
                    }
                    Err(error) => {

                        self.log("wasm", format!("failed: membrane_guest_init() ERROR: {:?}",error).as_str());
                        pass = false;
                    }
                }

            }
            Err(_) => {
                self.log("wasm", "failed: membrane_guest_init() [NOT REQUIRED]");
            }
        }

        {
            let test = "Test write string";
            match self.write_string(test){
                Ok(_) => {

                    self.log("wasm", "passed: write_string()");
                },
                Err(e) => {
                    self.log("wasm", format!("failed: write_string() test {:?}", e).as_str());
                    pass = false;

                }
            };
        }

        match pass{
            true => Ok(()),
            false => Err("init failed".into())
        }

    }

    pub fn log( &self, log_type:&str, message: &str )
    {
        println!("{} : {}",log_type,message);
    }

    pub fn write_string(&self, string: &str )->Result<i32,Error>
    {
        let string = string.as_bytes();
        let memory = self.instance.exports.get_memory("memory")?;
        let buffer_id = self.alloc_buffer(string.len() as _ )?;
        let buffer_ptr = self.get_buffer_ptr(buffer_id)?;
        let values = buffer_ptr.deref(memory, 0, string.len() as u32).unwrap();
        for i in 0..string.len() {
            values[i].set(string[i] );
        }

        Ok(buffer_id)
    }

    pub fn write_buffer(&self, bytes: &Vec<u8> )->Result<i32,Error>
    {
        let memory = self.instance.exports.get_memory("memory")?;
        let buffer_id = self.alloc_buffer(bytes.len() as _ )?;
        let buffer_ptr = self.get_buffer_ptr(buffer_id)?;
        let values = buffer_ptr.deref(memory, 0, bytes.len() as u32).unwrap();
        for i in 0..bytes.len() {
            values[i].set(bytes[i] );
        }

        Ok(buffer_id)
    }


    fn alloc_buffer(&self, len: i32 ) ->Result<i32,Error>
    {
        let buffer_id= self.instance.exports.get_native_function::<i32,i32>("membrane_guest_alloc_buffer").unwrap().call(len.clone())?;
        Ok(buffer_id)
    }

    fn get_buffer_ptr( &self, buffer_id: i32 )->Result<WasmPtr<u8,Array>,Error>
    {
        Ok(self.instance.exports.get_native_function::<i32, WasmPtr<u8, Array>>("membrane_guest_get_buffer_ptr").unwrap().call(buffer_id)?)
    }

    pub fn read_buffer(&self, buffer_id: i32 ) ->Result<Vec<u8>,Error>
    {
        let ptr = self.instance.exports.get_native_function::<i32,WasmPtr<u8,Array>>("membrane_guest_get_buffer_ptr").unwrap().call(buffer_id )?;
        let len = self.instance.exports.get_native_function::<i32,i32>("membrane_guest_get_buffer_len").unwrap().call(buffer_id )?;
        let memory = self.instance.exports.get_memory("memory")?;
        let values = ptr.deref(memory, 0, len as u32).unwrap();
        let mut rtn = vec!();
        for i in 0..values.len() {
           rtn.push( values[i].get() )
        }

        Ok(rtn)
    }

    pub fn read_string(&self, buffer_id: i32 ) ->Result<String,Error>
    {
        let raw = self.read_buffer(buffer_id)?;
        let rtn = String::from_utf8(raw)?;

        Ok(rtn)
    }

    fn consume_string(&self, buffer_id: i32 ) ->Result<String,Error>
    {
        let raw = self.read_buffer(buffer_id)?;
        let rtn = String::from_utf8(raw)?;
        self.membrane_guest_dealloc_buffer(buffer_id)?;
        Ok(rtn)
    }

    fn membrane_guest_dealloc_buffer( &self, buffer_id: i32 )->Result<(),Error>
    {
        self.instance.exports.get_native_function::<i32,()>("membrane_guest_dealloc_buffer")?.call(buffer_id.clone())?;
        Ok(())
    }


    pub fn test_panic(&self)->Result<(),Error>
    {
        self.instance.exports.get_native_function::<(),()>("wasm_test_panic").unwrap().call()?;
        Ok(())
    }


    pub fn test_log(&self)->Result<(),Error>
    {
        let log_message_string = "Some Log Message";
        let log_message_buffer = self.write_string(log_message_string)?;
        self.instance.exports.get_native_function::<i32,()>("membrane_guest_test_log").unwrap().call(log_message_buffer)?;
        Ok(())
    }

    pub fn test_endless_loop(&self)->Result<(),Error>
    {
        self.instance.exports.get_native_function::<(),()>("membrane_guest_example_test_endless_loop").unwrap().call()?;
        Ok(())
    }


}

#[derive(Clone)]
pub struct WasmBuffer
{
    ptr: WasmPtr<u8,Array>,
    len: u32
}

impl WasmBuffer
{
   pub fn new( ptr: WasmPtr<u8,Array>,
               len: u32 )->Self
   {
       WasmBuffer{
           ptr: ptr,
           len: len
       }
   }
}




struct WasmHost {
    membrane: Option<Weak<WasmMembrane>>,
}


impl  WasmHost{

    fn new() ->Self
    {
        WasmHost{
            membrane: Option::None,
        }
    }

}

#[derive(WasmerEnv, Clone)]
struct Env {
    host: Arc<RwLock<WasmHost>>,
}

impl Env
{
    pub fn unwrap(&self) -> Result<Arc<WasmMembrane>, Error>
    {
        let host = self.host.read();
        if host.is_err()
        {
            println!("WasmMembrane: could not acquire host lock");
            return Err("could not acquire host lock".into());
        }

        let host = host.unwrap();

        let membrane = host.membrane.as_ref();
        if membrane.is_none()
        {
            println!("WasmMembrane: membrane is not set");
            return Err("membrane is not set".into());
        }
        let membrane = membrane.unwrap().upgrade();

        if membrane.is_none()
        {
            println!("WasmMembrane: could not upgrade membrane reference");
            return Err("could not upgrade membrane reference".into());
        }
        let membrane = membrane.unwrap();
        let memory = membrane.instance.exports.get_memory("memory");
        if memory.is_err()
        {
            println!("WasmMembrane: could not access wasm memory");
            return Err("could not access wasm memory".into());
        }
        Ok(membrane)
    }
}

impl WasmMembrane {
    pub fn new(module: Arc<Module>) -> Result<Arc<Self>, Error> {
        let host = Arc::new(RwLock::new(WasmHost::new()));

        let imports = imports! { "env"=>{
        "membrane_host_log"=>Function::new_native_with_env(module.store(),Env{host:host.clone()},|env:&Env,buffer:i32| {
                match env.unwrap()
                {
                   Ok(membrane)=>{
                        let message = membrane.consume_string(buffer).unwrap_or("LOG ERROR".to_string());
                        membrane.log("guest",message.as_str());
                   },
                   Err(_)=>{}
                }
            }),

        "membrane_host_panic"=>Function::new_native_with_env(module.store(),Env{host:host.clone()},|env:&Env,buffer_id:i32| {
                match env.unwrap()
                {
                   Ok(membrane)=>{
                      let panic_message = membrane.consume_string(buffer_id).unwrap();
                      println!("WASM PANIC: {}",panic_message);
                   },
                   Err(_)=>{
                   println!("error panic");
                   }
                }
            }),
        } };


        let instance = Instance::new(&module, &imports)?;

        let membrane = Arc::new(WasmMembrane {
            instance: instance,
            //host: host.clone()
        });

        {
            host.write().unwrap().membrane = Option::Some(Arc::downgrade(&membrane));
        }

        return Ok(membrane);
    }
}

pub struct BufferLock
{
    id: i32,
    membrane: Arc<WasmMembrane>
}

impl BufferLock
{
    pub fn new( id: i32, membrane: Arc<WasmMembrane> )->Self
    {
        BufferLock{
           id: id,
           membrane: membrane
        }
    }

    pub fn id(&self)->i32
    {
        self.id.clone()
    }

    pub fn release(&self) -> Result<(),Error>
    {
        self.membrane.membrane_guest_dealloc_buffer(self.id)?;
        Ok(())
    }
}

impl Drop for BufferLock
{
    fn drop(&mut self) {
        self.release().unwrap_or(());
    }
}


#[cfg(test)]
mod test
{
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;
    use crate::membrane::WasmMembrane;
    use crate::error::Error;
    use wasmer::{Store, JIT, Cranelift, Module};
    use std::env;


    fn membrane() -> Result<Arc<WasmMembrane>, Error>
    {
        println!("CURRENT DIR {:?}", env::current_dir()? );
        let path = "../../../guest/rust/wasm_membrane_guest_example/pkg/wasm_membrane_bg.wasm";

        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let store = Store::new(&JIT::new(Cranelift::default()).engine());
        let module = Module::new(&store, data)?;
        let membrane = WasmMembrane::new(Arc::new(module)).unwrap();
        membrane.init()?;

        Ok(membrane)
    }


    #[test]
    pub fn test_wasm() -> Result<(), Error>
    {
        let membrane = membrane()?;

        let buffer_id = membrane.write_string("Hello From MEMBRANE!")?;

        membrane.membrane_guest_dealloc_buffer(buffer_id)?;

        Ok(())
    }


    #[test]
    pub fn test_log() -> Result<(), Error>
    {
        let membrane = membrane()?;
        membrane.test_log()?;

        Ok(())
    }

    #[test]
    pub fn test_endless_loop() -> Result<(), Error>
    {
        let membrane = membrane()?;
        membrane.test_endless_loop()?;

        Ok(())
    }


}


use car::aadlbook_integration::*;
use car::common_traits::*;

pub fn boot<S: System>() {
    let system = S::new();
    system.run();

    // 主线程保持运行，防止退出
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
fn main(){
    boot::<integrationSystem>();
}

// fn main(){
    
// }

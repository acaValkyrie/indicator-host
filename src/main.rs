use std::{
    fs::{self, DirEntry}, os::{linux::raw::stat, unix::process::ExitStatusExt}, path::PathBuf, sync::mpsc::RecvTimeoutError
};
use std::error::Error;
use std::io::prelude::*;
use std::time::Duration;
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;

fn list_dir(_path:&str) -> Vec<String> {
    let dir = PathBuf::from(_path);
    let entries: Vec<DirEntry> = fs::read_dir(dir).unwrap().filter_map(|f| f.ok()).collect();
    let mut dirs: Vec<String> = Vec::new();
    entries.iter().for_each(|e|
        dirs.push(e.file_name().into_string().unwrap())
    );
    dirs
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut port = serialport::new("/dev/indicator", 9600)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(100))
        .open()?;

    loop {
        // ディレクトリ一覧を取得
        let dirs = list_dir("/dev/");
        let mut peripherals = BTreeMap::new();
        peripherals.insert("uart_pico", false);
        for peripheral in peripherals.iter_mut(){
            let (key, value) = peripheral;
            let key_name = key.to_string();
            *value = dirs.contains(&key_name);
        }
        
        let status = Command::new("sh").arg("-c")
                        .arg("ping 192.168.1.169 -c 1").status().unwrap();
        let exist_lidar = status.success();
        peripherals.insert("lidar", exist_lidar);
        
        // ディレクトリ一覧を取得
        let input_dirs = list_dir("/dev/input");
        let mut inputs = BTreeMap::new();
        inputs.insert("js0", false);
        // peripherals.insert("ydlidar", false);
        for input in inputs.iter_mut(){
            let (key, value) = input;
            let key_name = key.to_string();
            *value = input_dirs.contains(&key_name);
        }
        // print!("{:?}", peripherals);
        // println!("{:?}", inputs);
        
        // 送信用コマンド作成
        let mut send_command = String::new();
        for peripheral in peripherals.iter(){
            let (key, value) = peripheral;
            if value == &true{
                send_command.push_str("1");
            }else{
                send_command.push_str("0");
            }
        }
        for input in inputs.iter(){
            let (key, value) = input;
            if value == &true{
                send_command.push_str("1");
            }else{
                send_command.push_str("0");
            }
        }
        send_command.push_str("\n");

        // シリアルで送信
        match port.write(send_command.as_bytes()){
            Ok(_) => std::io::stdout().flush()?,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            Err(e) => {eprintln!("{:?}", e); return Err(e.into());},
        }

        // sleep 1 sec
        std::thread::sleep(Duration::from_secs(1));
    }
}


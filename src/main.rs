#![windows_subsystem = "windows"]

use std::{mem, net::Ipv4Addr, time::Duration};

use fltk::{app::{self}, button::{Button, RadioRoundButton}, enums::Color, frame::Frame, group::Group, prelude::*, window::Window};
use fltk_evented::AsyncListener;
use winapi::{ctypes::c_void, um::{icmpapi::{IcmpCreateFile, IcmpParseReplies, IcmpSendEcho2}, ipexport::IP_OPTION_INFORMATION32, synchapi::{CreateEventExA, ResetEvent, WaitForSingleObjectEx}}};

#[allow(dead_code)]
async unsafe fn ping_test(label: &mut Frame, ip_addr: Ipv4Addr, pings: &mut Vec<i32>) {
    let packet: [u8; 512] = [0; 512];
    let packet_pointer: *const [u8; 512] = &packet;
    let mut resp = [0; 1024];
    let mut options = IP_OPTION_INFORMATION32::default();
    options.Ttl = 224;
    options.Flags = 0;
    let icmp_file = IcmpCreateFile();
    let event = CreateEventExA(std::ptr::null_mut(), std::ptr::null_mut(), 0x00000001, 0x1F0003);
    for _ in 0..10 {
        let _ = IcmpSendEcho2(
            icmp_file, 
            event,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            mem::transmute(ip_addr), 
            packet_pointer as *mut c_void, 
            512,
            &mut options, 
            &mut resp as *mut _ as *mut c_void, 
            1024, 
            2000);
        while WaitForSingleObjectEx(event, 50, 0) != 0 {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        IcmpParseReplies(&mut resp as *mut _ as *mut c_void, 1024);
        if resp[1] != 0 {
            pings.push(-1);
        }else {
            pings.push(resp[2]);
        }
        println!("{:?}",resp);
        
        resp = [0; 1024];
        ResetEvent(event);
        println!("{:?}",pings);
        label.set_label(&format!("Min : {}ms\nAvg : {}ms\nMax : {}ms\nPacket loss : {}%",pings.iter().min().unwrap(), (pings.iter().sum::<i32>()/pings.len() as i32), pings.iter().max().unwrap(), 0));
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

unsafe fn ping_test_blocking(label: &mut Frame, ip_addr: Ipv4Addr) {
    let mut pings: Vec<i32> = Vec::new();
    let packet: [u8; 512] = [0; 512];
    let packet_pointer: *const [u8; 512] = &packet;
    let mut resp = [0; 1024];
    let mut options = IP_OPTION_INFORMATION32::default();
    options.Ttl = 224;
    options.Flags = 0;
    let icmp_file = IcmpCreateFile();
    let event = CreateEventExA(std::ptr::null_mut(), std::ptr::null_mut(), 0x00000001, 0x1F0003);
    for _ in 0..25 {
        let _ = IcmpSendEcho2(
            icmp_file, 
            event,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            mem::transmute(ip_addr), 
            packet_pointer as *mut c_void, 
            512,
            &mut options, 
            &mut resp as *mut _ as *mut c_void, 
            1024, 
            2000);
        while WaitForSingleObjectEx(event, 100, 0) != 0 {
            std::thread::sleep(Duration::from_millis(100));
        }
        IcmpParseReplies(&mut resp as *mut _ as *mut c_void, 1024);
        if resp[1] != 0 {
            pings.push(-1);
        }else {
            pings.push(resp[2]);
        }        
        resp = [0; 1024];
        ResetEvent(event);
        println!("{:?}",pings);
        label.set_label(&format!("Pings completed : {}\nMin : {}ms\nAvg : {}ms\nMax : {}ms\nPackets lost : {}", pings.len(), pings.iter().filter(|x| {x != &&-1}).min().unwrap(), (pings.iter().filter(|x| {x != &&-1}).sum::<i32>()/pings.len() as i32), pings.iter().max().unwrap(), pings.iter().filter(|x| {x == &&-1}).count()));
        label.redraw_label();
        std::thread::sleep(Duration::from_millis(600));
    }
}


#[tokio::main]
async fn main() {
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 700, 400, "LoL ping test");
    wind.set_color(Color::from_rgb(255, 255, 255));
    let mut radio_group = Group::new(16, 16, 128, 300, "Server selection");
    let eune = RadioRoundButton::new(16, 32, 128, 32, "EUNE");
    let euw = RadioRoundButton::new(16, 64, 128, 32, "EUW");
    let br = RadioRoundButton::new(16, 96, 128, 32, "BR");
    let na = RadioRoundButton::new(16, 128, 128, 32, "NA");
    let oce = RadioRoundButton::new(16, 160, 128, 32, "OCE");
    let ru = RadioRoundButton::new(16, 192, 128, 32, "RU");
    let tr = RadioRoundButton::new(16, 224, 128, 32, "TR");
    radio_group.add(&eune);
    radio_group.add(&euw);
    radio_group.add(&br);
    radio_group.add(&na);
    radio_group.add(&oce);
    radio_group.add(&ru);
    radio_group.add(&tr);
    radio_group.end();
    let mut start_ping: AsyncListener<_> = Button::new(200, 16, 200, 64, "Start pinging").into();
    let mut label = Frame::new(200, 128, 200, 128, "");
    wind.end();
    wind.show();
    while app.wait() {
        if start_ping.triggered().await {
            unsafe {
                start_ping.deactivate();
                let ip_addr: Ipv4Addr;
                if eune.is_toggled() {
                    ip_addr = "52.94.17.106".parse::<Ipv4Addr>().unwrap()
                }else if euw.is_toggled() {
                    ip_addr = "35.71.111.131".parse::<Ipv4Addr>().unwrap()
                }else if br.is_toggled() {
                    ip_addr = "52.94.7.12".parse::<Ipv4Addr>().unwrap()
                }else if na.is_toggled() {
                    ip_addr = "108.60.126.230".parse::<Ipv4Addr>().unwrap()
                }else if oce.is_toggled() {
                    ip_addr = "52.94.11.140".parse::<Ipv4Addr>().unwrap()
                }else if ru.is_toggled() { // russian servers are located in germany, as well as the eune servers
                    ip_addr = "52.94.17.106".parse::<Ipv4Addr>().unwrap()
                }else if tr.is_toggled() {
                    ip_addr = "185.248.14.4".parse::<Ipv4Addr>().unwrap()
                }else {
                    label.set_label("No button selected");
                    return;
                }
                let mut clone_start_ping = start_ping.clone();
                let mut label = label.clone();
                std::thread::spawn(move || {
                    ping_test_blocking(&mut label, ip_addr);
                    clone_start_ping.activate();
                });

            }
        }
    }
    app.run().unwrap();
}

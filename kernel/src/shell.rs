//! Kernel shell

use alloc::string::String;
use alloc::vec::Vec;
use crate::fs::{ROOT_INODE, INodeExt};
use crate::process::*;

pub fn run_user_shell() {
    if let Ok(inode) = ROOT_INODE.lookup("sh") {
        println!("Going to user mode shell.");
        println!("Use 'ls' to list available programs.");
        processor().manager().add(Process::new_user(&inode, "sh".split(' ')), 0);
    } else {
        processor().manager().add(Process::new_kernel(shell, 0), 0);
    }
}

pub extern fn shell(_arg: usize) -> ! {
    let files = ROOT_INODE.list().unwrap();
    println!("Available programs: {:?}", files);

    loop {
        print!(">> ");
        let cmd = get_line();
        if cmd == "" {
            continue;
        }
        let name = cmd.split(' ').next().unwrap();
        if let Ok(inode) = ROOT_INODE.lookup(name) {
            let pid = processor().manager().add(Process::new_user(&inode, cmd.split(' ')), thread::current().id());
            unsafe { thread::JoinHandle::<()>::_of(pid) }.join().unwrap();
        } else {
            println!("Program not exist");
        }
    }
}

fn get_line() -> String {
    let mut s = String::new();
    loop {
        let c = get_char();
        match c {
            '\u{7f}' /* '\b' */ => {
                if s.pop().is_some() {
                    print!("\u{7f}");
                }
            }
            ' '...'\u{7e}' => {
                s.push(c);
                print!("{}", c);
            }
            '\n' | '\r' => {
                print!("\n");
                return s;
            }
            _ => {}
        }
    }
}

fn get_char() -> char {
    crate::fs::STDIN.pop()
}
